use ahash::{AHashMap, AHashSet};
use aocluster::{
    alg::{self, CCLabels},
    aoc::rayon::{
        self,
        prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    },
    belinda::{
        ClusteringHandle, ClusteringSource, EnrichedGraph, GraphStats, RichCluster, RichClustering,
    },
    utils::{calc_cpm_resolution, calc_modularity_resolution},
    DefaultGraph,
};
use arrow::bitmap::Bitmap;
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use polars::prelude::*;
use polars::{df, export::once_cell::sync::OnceCell};

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use roaring::{MultiOps, RoaringBitmap, RoaringTreemap};
use std::{collections::HashMap, io::BufReader, path::Path, sync::Arc};

use crate::{
    df::{
        build_series_from_bitmap, build_series_from_sets, iter_roaring, EfficientSet,
        VecEfficientSet,
    },
    ffi::{self, translate_df},
};

#[pyfunction]
pub fn set_nthreads(nthreads: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(nthreads)
        .build_global()
        .unwrap();
}

#[derive(Debug, Clone, PartialEq, Hash)]
#[pyclass]
pub enum SingletonMode {
    AutoPopulate,
    Ignore,
    AsIs,
}

pub fn populate_clusdf(g: &Graph, df: &mut DataFrame) -> anyhow::Result<()> {
    let g = &g.data.graph;
    let bitmaps: Vec<RoaringBitmap> = iter_roaring(df.column("nodes")?)
        .map(|n| n.try_into().unwrap())
        .collect_vec();
    let edges_bitmaps = g.nodes.iter().map(|n| RoaringBitmap::from_sorted_iter(n.edges.iter().map(|it| *it as u32)).unwrap()).collect_vec();
    let n = bitmaps.len();
    let data: Vec<_> = bitmaps
        .into_par_iter()
        .map(|nodes| {
            let mut m = 0u64;
            let mut c = 0u64;
            let mut mcd = (g.m() + 1) as u64;
            for u in nodes.iter() {
                let adj = &edges_bitmaps[u as usize];
                let ic = adj.intersection_len(&nodes);
                m += ic;
                c += adj.len() as u64 - ic;
                mcd = mcd.min(ic);
            }
            if mcd == (g.m() + 1) as u64 {
                mcd = 0;
            }
            m /= 2;
            (nodes.len(), m, c, mcd)
        })
        .collect();
    let mut n_s = Vec::with_capacity(data.len());
    let mut m_s = Vec::with_capacity(data.len());
    let mut c_s = Vec::with_capacity(data.len());
    let mut mcd_s = Vec::with_capacity(data.len());
    for (n, m, c, mcd) in data {
        n_s.push(n);
        m_s.push(m);
        c_s.push(c);
        mcd_s.push(mcd);
    }
    df.with_column(Series::new("n", n_s))?;
    df.with_column(Series::new("m", m_s))?;
    df.with_column(Series::new("c", c_s))?;
    df.with_column(Series::new("mcd", mcd_s))?;
    Ok(())
}

pub fn read_json<P: AsRef<Path>>(
    g: &Graph,
    filepath: P,
    mode: SingletonMode,
) -> anyhow::Result<DataFrame> {
    let mut file = std::fs::File::open(filepath)?;
    let mut df = JsonLineReader::new(&mut file).finish()?;
    df.with_column(
        df.column("nodes")?
            .cast(&DataType::List(Box::new(DataType::UInt32)))?,
    )?;
    let mut nodes = node_list_to_bitmaps(g, df.column("nodes")?)?;
    nodes.rename("nodes");
    df.with_column(nodes)?;
    df = post_read_singleton(g, df, mode)?;
    populate_clusdf(g, &mut df)?;
    Ok(df)
}

pub fn post_read_singleton(
    g: &Graph,
    mut df: DataFrame,
    mode: SingletonMode,
) -> anyhow::Result<DataFrame> {
    if mode == SingletonMode::Ignore {
        let mask: Series = df
            .column("nodes")?
            .list()?
            .into_iter()
            .map(|f| f.map_or(false, |e| e.len() > 1))
            .collect();
        df = df.filter(mask.bool()?)?;
    }
    if mode == SingletonMode::AutoPopulate {
        let covered_nodes = iter_roaring(df.column("nodes")?).collect_vec().union();
        let covered_nodes: RoaringBitmap = covered_nodes.try_into()?;
        let mut new_labels = vec![];
        let mut new_nodes: Vec<EfficientSet> = vec![];
        if covered_nodes.len() < g.n().into() {
            for i in 0..g.n() {
                if !covered_nodes.contains(i) {
                    new_labels.push(AnyValue::Null);
                    new_nodes.push(RoaringBitmap::from_iter([i]).into())
                }
            }
        }
        let new_labels =
            Series::from_any_values_and_dtype("label", &new_labels, df.column("label")?.dtype())?;
        let extend_df = df!("label" => new_labels, "nodes" => new_nodes.to_series())?;
        df.extend(&extend_df)?;
    }
    Ok(df)
}

pub fn read_assignment_series(
    g: &Graph,
    nodes: &Series,
    cids: &Series,
    mode: SingletonMode,
) -> anyhow::Result<DataFrame> {
    let df = df!("nid" => nodes.cast(&DataType::UInt32)?, "cid" => cids)?;
    let mut df = df
        .lazy()
        .groupby(["cid"])
        .agg([col("nid").list()])
        .collect()?;
    // println!("collected");
    if mode == SingletonMode::Ignore {
        let mask: Series = df
            .column("nid")?
            .list()?
            .into_iter()
            .map(|f| f.map_or(false, |e| e.len() > 1))
            .collect();
        df = df.filter(mask.bool()?)?;
    }
    let mut nodes = node_list_to_bitmaps(g, df.column("nid")?)?;
    nodes.rename("nodes");
    // println!("nodes");
    let mut df = df!("label" => df.column("cid")?, "nodes" => nodes)?;
    if mode == SingletonMode::AutoPopulate {
        let covered_nodes = iter_roaring(df.column("nodes")?).collect_vec().union();
        let covered_nodes: RoaringBitmap = covered_nodes.try_into()?;
        let mut new_labels = vec![];
        let mut new_nodes: Vec<EfficientSet> = vec![];
        if covered_nodes.len() < g.n().into() {
            for i in 0..g.n() {
                if !covered_nodes.contains(i) {
                    new_labels.push(AnyValue::Null);
                    new_nodes.push(RoaringBitmap::from_iter([i]).into())
                }
            }
        }
        let new_labels =
            Series::from_any_values_and_dtype("label", &new_labels, df.column("label")?.dtype())?;
        let extend_df = df!("label" => new_labels, "nodes" => new_nodes.to_series())?;
        df.extend(&extend_df)?;
    }
    populate_clusdf(g, &mut df)?;
    Ok(df)
}

pub fn read_assignment_file(
    g: &Graph,
    filepath: &str,
    sep: u8,
    mode: SingletonMode,
) -> anyhow::Result<DataFrame> {
    let df = CsvReader::from_path(filepath)?
        .has_header(false)
        .with_delimiter(sep)
        .finish()?;
    let nid = df.column("column_1")?;
    let cid = df.column("column_2")?;
    read_assignment_series(g, nid, cid, mode)
}

#[pyfunction(
    name = "read_assignment",
    mode = "SingletonMode::AutoPopulate",
    sep = "b'\\t'"
)]
pub fn py_read_assignment_file(
    g: &Graph,
    filepath: &str,
    sep: u8,
    mode: SingletonMode,
) -> PyResult<PyObject> {
    let mut df = read_assignment_file(g, filepath, sep, mode).unwrap();
    let translated = translate_df(&mut df)?;
    Ok(translated)
}

#[pyfunction(name = "read_assignment_series", mode = "SingletonMode::AutoPopulate")]
pub fn py_from_assignments(
    g: &Graph,
    nodes: &PyAny,
    cids: &PyAny,
    mode: SingletonMode,
) -> PyResult<PyObject> {
    let nodes = ffi::py_series_to_rust_series(nodes)?;
    let cids = ffi::py_series_to_rust_series(cids)?;
    let mut df = read_assignment_series(g, &nodes, &cids, mode).unwrap();
    Ok(translate_df(&mut df)?)
}

#[pyfunction(name = "read_json", mode = "SingletonMode::AutoPopulate")]
pub fn py_read_json(g: &Graph, filepath: &str, mode: SingletonMode) -> PyResult<PyObject> {
    let mut df = read_json(g, filepath, mode).unwrap();
    Ok(translate_df(&mut df)?)
}

pub fn node_list_to_bitmaps(g: &Graph, list: &Series) -> anyhow::Result<Series> {
    let g = &g.data.graph;
    let as_list = list.list()?;
    let sets: Vec<EfficientSet> = as_list
        .par_iter()
        .map(|e| {
            e.map_or_else(
                || RoaringBitmap::new().into(),
                |series| {
                    let mut seen_nonexistent = false;
                    let mut bitmap = RoaringBitmap::new();
                    series
                        .u32()
                        .unwrap()
                        .into_iter()
                        .filter_map(|x| x)
                        .for_each(|x| match g.retrieve(x as usize) {
                            Some(internal_id) => {
                                bitmap.insert(internal_id as u32);
                            }
                            None => {
                                if seen_nonexistent {
                                    panic!("Nonexistent node that is not singleton: {}", x);
                                }
                                seen_nonexistent = true;
                            }
                        });
                    bitmap.into()
                },
            )
        })
        .collect();
    Ok(sets.to_series())
}

// #[pyfunction(with_singletons = "true")]
// pub fn read_clusters(
//     py: Python,
//     g: &Graph,
//     clus_path: &str,
//     with_singletons: bool,
// ) -> PyResult<PyObject> {
//     let clus = Clustering::new(py, g, clus_path, None)?;
//     let mut df = df!(
//         "label" => clus.data.clusters.keys().copied().collect_vec(),
//         "n" => clus.data.clusters.values().map(|v| v.n as u32).collect_vec(),
//         "m" => clus.data.clusters.values().map(|v| v.m).collect_vec(),
//         "c" => clus.data.clusters.values().map(|v| v.c).collect_vec(),
//         "mcd" => clus.data.clusters.values().map(|v| v.mcd).collect_vec(),
//         "nodes" => build_series_from_bitmap(clus.data.clusters.values().map(|v| v.nodes.clone()).collect_vec()),
//     ).unwrap();
//     translate_df(&mut df)
// }

#[pyclass]
#[derive(Clone)]
pub struct Graph {
    data: Arc<EnrichedGraph>,
    cc: OnceCell<CCLabels>,
}

pub trait ClusDataFrame {
    fn modularity(&self, graph: &Graph, resolution: f64) -> anyhow::Result<Series>;
    fn cpm(&self, resolution: f64) -> anyhow::Result<Series>;
    fn covered_num_nodes(&self) -> anyhow::Result<u32>;
    fn edges(&self, graph: &DefaultGraph) -> anyhow::Result<Series>;
    fn covered_edges(&self, graph: &DefaultGraph) -> anyhow::Result<Series>;
    fn can_overlap(&self, graph: &DefaultGraph) -> bool;
}

impl ClusDataFrame for DataFrame {
    fn modularity(&self, graph: &Graph, resolution: f64) -> anyhow::Result<Series> {
        let m = self.column("m")?.u64()?;
        let c = self.column("c")?.u64()?;
        let total_l = graph.m();
        Ok((m.into_iter())
            .zip(c.into_iter())
            .map(|(m, c)| {
                let vol = 2 * m.unwrap() + c.unwrap();
                calc_modularity_resolution(
                    m.unwrap() as usize,
                    vol as usize,
                    total_l as usize,
                    resolution,
                )
            })
            .collect())
    }

    fn cpm(&self, resolution: f64) -> anyhow::Result<Series> {
        let n = self.column("n")?.u32()?;
        let m = self.column("m")?.u64()?;
        Ok(n.into_iter()
            .zip(m.into_iter())
            .map(|(n, m)| calc_cpm_resolution(m.unwrap() as usize, n.unwrap() as usize, resolution))
            .collect())
    }

    fn covered_num_nodes(&self) -> anyhow::Result<u32> {
        self.column("can_overlap")
            .and_then(|_can_overlap| {
                let nodesets = self.column("nodes")?;
                let nodesets = iter_roaring(nodesets)
                    .map(|it| it.try_into().unwrap())
                    .collect::<Vec<RoaringBitmap>>();
                Ok(nodesets.union().len() as u32)
            })
            .or_else(|_| {
                let n = self.column("n")?.u32()?;
                Ok(n.into_iter().map(|n| n.unwrap()).sum())
            })
    }

    fn edges(&self, _graph: &DefaultGraph) -> anyhow::Result<Series> {
        todo!()
    }

    fn covered_edges(&self, _graph: &DefaultGraph) -> anyhow::Result<Series> {
        todo!()
    }

    fn can_overlap(&self, _graph: &DefaultGraph) -> bool {
        todo!()
    }
}

impl Graph {
    pub fn get_cc_labels(&self) -> &CCLabels {
        self.cc.get_or_init(|| alg::cc_labeling(&self.data.graph))
    }
}

#[pymethods]
impl Graph {
    #[new]
    fn new(filepath: &str) -> Self {
        let raw_data =
            EnrichedGraph::from_graph(aocluster::base::Graph::parse_from_file(filepath).unwrap());
        Graph {
            data: Arc::new(raw_data),
            cc: OnceCell::new(),
        }
    }

    #[args(verbose = false)]
    fn nodes(&self, clus: Option<&PyAny>, verbose: bool) -> PyResult<PyObject> {
        let g = &self.data.graph;
        let nodes = (0..self.n())
            .map(|it| g.name_set.rev[it as usize] as u32)
            .collect_vec();
        let degrees = (0..self.n())
            .map(|it| g.nodes[it as usize].degree() as u32)
            .collect_vec();
        let mut df = df!(
            "node" => nodes,
            "degree" => degrees,
        )
        .unwrap();
        if verbose {
            let adj = (0..self.n())
                .map(|it| {
                    g.nodes[it as usize]
                        .edges
                        .iter()
                        .map(|it| g.name_set.rev[*it] as u32)
                        .collect::<Series>()
                })
                .collect_vec();
            df.with_column(Series::new("adj", adj)).unwrap();
        }
        if let Some(clus) = clus {
            let label =
                ffi::py_series_to_rust_series(clus.call_method1("get_column", ("label",))?)?;
            let label_t = label.dtype();
            let mut labels_u32: Vec<Vec<Option<u32>>> = vec![vec![]; self.n() as usize];
            let mut labels_str: Vec<Vec<String>> = vec![vec![]; self.n() as usize];
            let nodes =
                ffi::py_series_to_rust_series(clus.call_method1("get_column", ("nodes",))?)?;
            if label_t != &DataType::Utf8 {
                for (ns, label) in
                    iter_roaring(&nodes).zip(label.cast(&DataType::UInt32).unwrap().u32().unwrap())
                {
                    let ns: RoaringBitmap = ns.try_into().unwrap();
                    for node in ns.into_iter() {
                        labels_u32[node as usize].push(label);
                    }
                }
            } else {
                for (ns, label) in iter_roaring(&nodes).zip(label.utf8().unwrap()) {
                    let ns: RoaringBitmap = ns.try_into().unwrap();
                    for node in ns.into_iter() {
                        labels_str[node as usize].push(label.unwrap_or_default().to_string());
                    }
                }
            }
            let labels_u32 = labels_u32
                .into_iter()
                .map(|it| it.into_iter().collect::<Series>())
                .collect_vec();
            let labels_str = labels_str
                .into_iter()
                .map(|it| it.into_iter().collect::<Series>())
                .collect_vec();
            if label_t != &DataType::Utf8 {
                df.with_column(Series::new("labels", labels_u32)).unwrap();
            } else {
                df.with_column(Series::new("labels", labels_str)).unwrap();
            }
        }
        Ok(translate_df(&mut df)?)
    }

    fn covered_edges(&self, n: &PyAny) -> PyResult<PyObject> {
        let series = ffi::py_series_to_rust_series(n)?;
        let g = &self.data;
        let nodesets = iter_roaring(&series)
            .map(|it| it.try_into().unwrap())
            .map(|it| edgeset(g, &it))
            .map(EfficientSet::BigSet)
            .collect::<Vec<_>>();
        ffi::rust_series_to_py_series(&build_series_from_sets(nodesets))
    }

    #[getter]
    fn n(&self) -> u32 {
        self.data.graph.n() as u32
    }

    #[getter]
    fn m(&self) -> u64 {
        self.data.graph.m() as u64
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Graph(n={}, m={})",
            self.data.graph.n(),
            self.data.graph.m()
        ))
    }

    fn num_components(&self) -> u32 {
        self.get_cc_labels().num_nodes.len() as u32
    }

    fn largest_component(&self) -> u32 {
        self.get_cc_labels()
            .num_nodes
            .iter()
            .max()
            .copied()
            .unwrap() as u32
    }
}

#[pyclass]
pub struct ClusterSkeleton {
    #[pyo3(get)]
    n: u64,
    #[pyo3(get)]
    m: u64,
    #[pyo3(get)]
    c: u64,
    #[pyo3(get)]
    mcd: u64,
    #[pyo3(get)]
    vol: u64,
}

#[pymethods]
impl ClusterSkeleton {
    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "ClusterSkeleton(n={}, m={}, c={})",
            self.n, self.m, self.c,
        ))
    }
}

impl From<RichCluster> for ClusterSkeleton {
    fn from(cluster: RichCluster) -> Self {
        ClusterSkeleton {
            n: cluster.n,
            m: cluster.m,
            c: cluster.c,
            mcd: cluster.mcd,
            vol: cluster.vol,
        }
    }
}

impl ClusterSkeleton {
    fn from_cluster(cluster: &RichCluster) -> Self {
        ClusterSkeleton {
            n: cluster.n,
            m: cluster.m,
            c: cluster.c,
            mcd: cluster.mcd,
            vol: cluster.vol,
        }
    }
}

#[pyclass]
pub struct Clustering {
    data: Arc<RichClustering<true>>,
}

#[pyclass]
pub struct ClusteringSubset {
    data: ClusteringHandle<true>,
}

#[pymethods]
impl Clustering {
    #[new]
    #[args(py_kwargs = "**")]
    fn new(
        py: Python,
        graph: &Graph,
        filepath: &str,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<Self> {
        let mut source = ClusteringSource::Unknown;
        if let Some(kwargs) = py_kwargs {
            if let Some(cpm_resolution) = kwargs.get_item("cpm") {
                source = ClusteringSource::Cpm(cpm_resolution.extract()?);
            }
        }
        let raw_data = py.allow_threads(move || {
            let mut clus =
                RichClustering::<true>::pack_from_file(graph.data.clone(), filepath).unwrap();
            clus.source = source;
            clus
        });
        Ok(Clustering {
            data: Arc::new(raw_data),
        })
    }

    fn __getitem__(&self, ids: &PyList) -> PyResult<ClusteringSubset> {
        let ids: Vec<u32> = ids.extract()?;
        let data = ClusteringSubset {
            data: ClusteringHandle::new(self.data.clone(), ids.into_iter().collect(), false),
        };
        Ok(data)
    }

    fn filter(&self, f: &PyAny) -> PyResult<ClusteringSubset> {
        let v = self
            .data
            .clusters
            .iter()
            .filter(|(_k, v)| {
                f.call((ClusterSkeleton::from_cluster(v),), None)
                    .unwrap()
                    .extract()
                    .unwrap()
            })
            .map(|(k, _v)| k)
            .copied()
            .collect();
        let has_singletons = f
            .call(
                (ClusterSkeleton {
                    n: 1,
                    m: 0,
                    c: 0,
                    mcd: 0,
                    vol: 0,
                },),
                None,
            )
            .unwrap()
            .extract()
            .unwrap();
        Ok(ClusteringSubset {
            data: ClusteringHandle::new(self.data.clone(), v, has_singletons),
        })
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Clustering(covered_nodes={}, size={})",
            self.data.cover.len(),
            self.data.clusters.len(),
        ))
    }

    pub fn size(&self) -> usize {
        self.data.clusters.len()
    }
}

#[pyclass(name = "ClusteringStats")]
pub struct StatsWrapper {
    #[pyo3(get)]
    num_clusters: u32,
    #[pyo3(get)]
    covered_nodes: u32,
    #[pyo3(get)]
    covered_edges: u64,
    #[pyo3(get)]
    total_nodes: u32,
    #[pyo3(get)]
    total_edges: u64,
    #[pyo3(get)]
    distributions: HashMap<String, SummarizedDistributionWrapper>,
}

impl StatsWrapper {
    pub fn from_graph_stats(graph_stats: GraphStats) -> Self {
        StatsWrapper {
            num_clusters: graph_stats.num_clusters,
            covered_nodes: graph_stats.covered_nodes,
            covered_edges: graph_stats.covered_edges,
            total_nodes: graph_stats.total_nodes,
            total_edges: graph_stats.total_edges,
            distributions: graph_stats
                .statistics
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.to_string().to_lowercase(),
                        SummarizedDistributionWrapper::new(v),
                    )
                })
                .collect(),
        }
    }
}

#[pyclass(name = "SummarizedDistribution")]
#[derive(Debug, Clone)]
pub struct SummarizedDistributionWrapper {
    data: aocluster::belinda::SummarizedDistribution,
}

impl SummarizedDistributionWrapper {
    fn new(data: aocluster::belinda::SummarizedDistribution) -> Self {
        SummarizedDistributionWrapper { data }
    }
}

#[pymethods]
impl SummarizedDistributionWrapper {
    #[getter]
    pub fn percentiles(&self) -> Vec<f64> {
        self.data.percentiles.to_vec()
    }

    #[getter]
    pub fn minimum(&self) -> f64 {
        self.data.minimum()
    }

    #[getter]
    pub fn maximum(&self) -> f64 {
        self.data.maximum()
    }

    #[getter]
    pub fn median(&self) -> f64 {
        self.data.median()
    }
}

// pub fn union_bitmaps<E: AsRef<[Expr]>>(exprs: E) -> Expr {
//     let exprs = exprs.as_ref().to_vec();

//     let function = SpecialEq::new(Arc::new(move |series: &mut [Series]| {
//         let mut s_iter = series.iter();

//         match s_iter.next() {
//             Some(acc) => {
//                 let mut acc = acc.clone();
//                 let bitmaps = iter_roaring(&acc)
//                     .map(|it| it.try_into().unwrap())
//                     .collect::<Vec<RoaringBitmap>>();
//                 let series = build_series_from_bitmap(vec![bitmaps.union()]);
//                 Ok(series)
//             }
//             None => Err(PolarsError::ComputeError(
//                 "Reduce did not have any expressions to fold".into(),
//             )),
//         }
//     }) as Arc<dyn SeriesUdf>);

//     Expr::AnonymousFunction {
//         input: exprs,
//         function,
//         output_type: GetOutput::super_type(),
//         options: FunctionOptions {
//             collect_groups: ApplyOptions::ApplyGroups,
//             input_wildcard_expansion: true,
//             auto_explode: true,
//             fmt_str: "reduce",
//             ..Default::default()
//         },
//     }
// }

pub fn rust_label_cc(g: &Graph, series: &Series) -> anyhow::Result<Series> {
    let labels = &g.get_cc_labels().labels;
    let g = &g.data.graph;
    let mut ans = vec![];
    for u in series.u32().into_iter() {
        for v in u {
            ans.push(v.map(|v| labels[g.retrieve(v as usize).unwrap() as usize]));
        }
    }
    Ok(Series::new("cc", ans))
}

pub fn rust_label_cc_size(g: &Graph, series: &Series) -> anyhow::Result<Series> {
    let num_nodes = &g.get_cc_labels().num_nodes;
    let mut ans = vec![];
    for u in series.u32().into_iter() {
        for v in u {
            ans.push(v.map(|v| num_nodes[v as usize]));
        }
    }
    Ok(Series::new("cc_size", ans))
}

pub fn rust_nodeset_to_list(g: &Graph, series: &Series) -> anyhow::Result<Series> {
    let mut ans = vec![];
    let g = &g.data.graph;
    for bm in iter_roaring(series) {
        let bm: RoaringBitmap = bm.try_into()?;
        let s = bm
            .iter()
            .map(|it| g.name_set.rev[it as usize] as u32)
            .collect::<Series>();
        ans.push(s);
    }
    Ok(Series::new("nodes_list", ans))
}

pub fn rust_popcnt(series: &Series) -> Series {
    iter_roaring(series)
        .map(|bitmap| bitmap.len() as u32)
        .collect()
}

pub fn rust_bitmap_union(series: &Series) -> Series {
    let s = iter_roaring(series).collect::<Vec<EfficientSet>>();
    build_series_from_sets(vec![s.union()])
}

fn edgeset(g: &EnrichedGraph, bm: &RoaringBitmap) -> RoaringTreemap {
    let graph = &g.graph;
    let acc = &g.acc_num_edges;
    let tm = RoaringTreemap::from_sorted_iter(bm.iter().flat_map(|u| {
        let edges = &graph.nodes[u as usize].edges;
        let shift = acc[u as usize];
        edges
            .iter()
            .filter(move |e| u < **e as u32)
            .enumerate()
            .filter_map(move |(offset, &v)| {
                if bm.contains(v as u32) {
                    Some(shift + offset as u64)
                } else {
                    None
                }
            })
    }))
    .unwrap();
    tm
}

// pub fn rust_edgeset(series: &Series) -> Series {
//     iter_roaring(series)
//         .map(|bitmap| bitmap.len() as u32)
//         .collect()
// }

#[pyfunction(name = "popcnt")]
pub fn py_popcnt(series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_popcnt(&series);
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "union")]
pub fn py_bitmap_union(series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_bitmap_union(&series);
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "cc_labels")]
pub fn py_label_cc(g: &Graph, series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_label_cc(&g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "cc_size")]
pub fn py_label_cc_size(g: &Graph, series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_label_cc_size(&g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "nodeset_to_list")]
pub fn py_nodeset_to_list(g: &Graph, series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_nodeset_to_list(&g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}

#[pymethods]
impl ClusteringSubset {
    fn compute_statistics(&self, py: Python) -> StatsWrapper {
        py.allow_threads(move || {
            let stats = self.data.stats();
            StatsWrapper::from_graph_stats(stats)
        })
    }

    fn __getitem__(&self, key: u32) -> PyResult<ClusterSkeleton> {
        let clus = &self.data.clustering;
        if let Some(cluster) = clus.clusters.get(&key) {
            Ok(ClusterSkeleton::from_cluster(cluster))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                "Cluster not found",
            ))
        }
    }

    fn to_df(&self) -> PyResult<PyObject> {
        let in_scope_clusters = self
            .data
            .cluster_ids
            .iter()
            .map(|it| &self.data.clustering.clusters[&it])
            .collect::<Vec<_>>();
        let mcd = in_scope_clusters
            .iter()
            .map(|it| it.mcd)
            .collect::<Vec<_>>();
        let mut df = df!("mcd" => mcd).unwrap();
        translate_df(&mut df)
    }

    fn keys(&self) -> Vec<u32> {
        self.data.cluster_ids.iter().collect()
    }

    fn size(&self) -> u64 {
        self.data.cluster_ids.len()
    }

    fn compute_size_diff(&self, rhs: &Clustering) -> (u32, SummarizedDistributionWrapper) {
        let (diff, dist) = self.data.size_diff(rhs.data.as_ref());
        (diff, SummarizedDistributionWrapper::new(dist))
    }

    #[getter]
    fn cluster_sizes(&self) -> Vec<u32> {
        let d = &self.data;
        d.cluster_ids
            .iter()
            .map(|k| d.clustering.clusters.get(&k).unwrap().nodes.len() as u32)
            .collect()
    }

    #[getter]
    fn node_coverage(&self) -> f64 {
        self.data.get_covered_nodes() as f64 / self.data.graph.graph.n() as f64
    }

    #[getter]
    fn num_singletons(&self) -> u32 {
        if self.data.has_singletons {
            self.data.clustering.singleton_clusters.len() as u32
        } else {
            0
        }
    }

    fn node_multiplicities(&self) -> Vec<u32> {
        let raw_mult = &self.data.node_multiplicity;
        let mut mults: Vec<_> = self
            .data
            .covered_nodes
            .iter()
            .map(|n| raw_mult[n as usize])
            .collect();
        if self.data.has_singletons {
            mults.extend((0..self.num_singletons()).map(|_| 1));
        }
        mults
    }

    #[getter]
    fn node_multiplicities_dist(&self) -> SummarizedDistributionWrapper {
        SummarizedDistributionWrapper::new(
            self.node_multiplicities()
                .into_iter()
                .map(|it| it as f64)
                .collect(),
        )
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "ClusteringSubset(size={}, node_coverage={:.1}%, is_overlapping={})",
            self.data.cluster_ids.len(),
            self.node_coverage() * 100.0,
            self.data.is_overlapping
        ))
    }
}

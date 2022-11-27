use aocluster::{
    aoc::rayon,
    belinda::{
        ClusteringHandle, ClusteringSource, EnrichedGraph, GraphStats, RichCluster, RichClustering,
    },
    utils::{calc_cpm_resolution, calc_modularity_resolution},
    DefaultGraph,
};
use itertools::Itertools;
use polars::prelude::*;
use polars::{df};

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use roaring::{MultiOps, RoaringBitmap, RoaringTreemap};
use std::{
    collections::HashMap,
    sync::{Arc},
};

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

#[pyfunction(with_singletons = "true")]
pub fn read_clusters(
    py: Python,
    g: &Graph,
    clus_path: &str,
    with_singletons: bool,
) -> PyResult<PyObject> {
    let clus = Clustering::new(py, g, clus_path, None)?;
    let mut df = df!(
        "label" => clus.data.clusters.keys().copied().collect_vec(),
        "n" => clus.data.clusters.values().map(|v| v.n as u32).collect_vec(),
        "m" => clus.data.clusters.values().map(|v| v.m).collect_vec(),
        "c" => clus.data.clusters.values().map(|v| v.c).collect_vec(),
        "mcd" => clus.data.clusters.values().map(|v| v.mcd).collect_vec(),
        "nodes" => build_series_from_bitmap(clus.data.clusters.values().map(|v| v.nodes.clone()).collect_vec()),
    ).unwrap();
    translate_df(&mut df)
}

#[pyclass]
#[derive(Clone)]
pub struct Graph {
    data: Arc<EnrichedGraph>,
}

pub trait ClusDataFrame {
    fn modularity(&self, graph: &Graph, resolution: f64) -> anyhow::Result<Series>;
    fn cpm(&self, resolution: f64) -> anyhow::Result<Series>;
    fn covered_num_nodes(&self) -> anyhow::Result<u32>;
    fn edges(&self, graph: &DefaultGraph) -> anyhow::Result<Series>;
    fn covered_edges(&self, graph: &DefaultGraph) -> anyhow::Result<Series>;
    fn can_overlap(&self, graph: &DefaultGraph) -> bool;
}

// fn series_cpm(n: &Series, m: &Series, resolution: f64) -> anyhow::Result<Series> {
//     let n = n.u32()?;
//     let m = m.u64()?;
//     Ok(n.into_iter()
//         .zip(m.into_iter())
//         .map(|(n, m)| calc_cpm_resolution(m.unwrap() as usize, n.unwrap() as usize, resolution))
//         .collect())
// }

// #[pyfunction]
// pub fn cpm(df: &PyAny, r: f64) -> PyResult<PyObject> {
//     let n_series = df.call_method1("get_column", ("n",))?;
//     let m_series = df.call_method1("get_column", ("m",))?;
//     let n_series = ffi::py_series_to_rust_series(n_series)?;
//     let m_series = ffi::py_series_to_rust_series(m_series)?;
//     let cpm_series = series_cpm(&n_series, &m_series, r).unwrap();
//     Ok(ffi::rust_series_to_py_series(&cpm_series)?)
// }

impl ClusDataFrame for DataFrame {
    fn modularity(&self, graph: &Graph, resolution: f64) -> anyhow::Result<Series> {
        // let n = self.column("n")?.u32()?;
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

#[pymethods]
impl Graph {
    #[new]
    fn new(filepath: &str) -> Self {
        let raw_data =
            EnrichedGraph::from_graph(aocluster::base::Graph::parse_from_file(filepath).unwrap());
        Graph {
            data: Arc::new(raw_data),
        }
    }

    fn covered_edges(&self, n: &PyAny) -> PyResult<PyObject> {
        let series = ffi::py_series_to_rust_series(n)?;
        let g = &self.data;
        let nodesets = iter_roaring(&series)
            .map(|it| it.try_into().unwrap())
            .map(|it| edgeset(g, &it))
            .map(EfficientSet::BigSet)
            .collect::<Vec<_>>();
        ffi::rust_series_to_py_series(&build_series_from_sets(
            nodesets,
        ))
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
            "Graph(n={}, m ={})",
            self.data.graph.n(),
            self.data.graph.m()
        ))
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

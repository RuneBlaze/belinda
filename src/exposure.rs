use aocluster::{
    alg::{self, CCLabels},
    aoc::rayon::{
        self,
        prelude::{
            IndexedParallelIterator, IntoParallelIterator, ParallelBridge, ParallelIterator,
        },
    },
    belinda::{
        EnrichedGraph,
    },
    utils::{calc_cpm_resolution, calc_modularity_resolution},
};
use itertools::Itertools;
use polars::prelude::*;
use polars::{df, export::once_cell::sync::OnceCell};

use pyo3::{
    prelude::*,
};
use roaring::{MultiOps, RoaringBitmap, RoaringTreemap};
use std::{path::Path, sync::Arc};

use crate::{
    df::{build_series_from_sets, iter_roaring, EfficientSet, VecEfficientSet},
    ffi::{self, translate_df},
};

#[pyfunction]
pub fn set_nthreads(nthreads: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(nthreads)
        .build_global()
        .unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    let edges_bitmaps = g
        .nodes
        .iter()
        .map(|n| RoaringBitmap::from_sorted_iter(n.edges.iter().map(|it| *it as u32)).unwrap())
        .collect_vec();
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
    df = postprocess_singleton_mode(g, df, mode)?;
    populate_clusdf(g, &mut df)?;
    Ok(df)
}

/// Postprocesses a data frame with the singleton mode specified
pub fn postprocess_singleton_mode(
    g: &Graph,
    mut df: DataFrame,
    mode: SingletonMode,
) -> anyhow::Result<DataFrame> {
    let lb = if mode == SingletonMode::Ignore {
        2
    } else {
        1
    };
    let mask: Series = iter_roaring(df.column("nodes")?)
            .map(|it| it.len() >= lb)
            .collect();
    df = df.filter(mask.bool()?)?;
    if mode == SingletonMode::AutoPopulate {
        let covered_nodes: RoaringBitmap = iter_roaring(df.column("nodes")?).collect_vec().union().try_into()?;
        // create two columns, a column of labels and a column of nodes
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
        let k = new_labels.len();
        let mut extend_df = df!("label" => new_labels, "nodes" => new_nodes.to_series())?;
        df.get_column_names_owned().iter().for_each(|col: &String| {
            if col != "label" && col != "nodes" {
                let mut null_filled = Vec::with_capacity(k);
                for _i in 0..k {
                    null_filled.push(AnyValue::Null);
                }
                let s = Series::from_any_values_and_dtype(
                    col,
                    &null_filled,
                    df.column(col).unwrap().dtype(),
                )
                .unwrap();
                extend_df.with_column(s).unwrap();
            }
        });
        df.extend(&extend_df)?;
    }
    Ok(df)
}

pub fn read_membership_series(
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
    let lb = if mode == SingletonMode::Ignore {
        2
    } else {
        1
    };
    let mask: Series = df
            .column("nid")?
            .list()?
            .into_iter()
            .map(|f| f.map_or(false, |e| e.len() >= lb))
            .collect();
    df = df.filter(mask.bool()?)?;
    let mut nodes = node_list_to_bitmaps(g, df.column("nid")?)?;
    nodes.rename("nodes");
    let mut df = df!("label" => df.column("cid")?, "nodes" => nodes)?;
    df = postprocess_singleton_mode(g, df, mode)?;
    populate_clusdf(g, &mut df)?;
    Ok(df)
}

pub fn read_membership_file(
    g: &Graph,
    filepath: &str,
    sep: u8,
    mode: SingletonMode,
    force_string_labels: bool,
) -> anyhow::Result<DataFrame> {
    let df = CsvReader::from_path(filepath)?
        .has_header(false)
        .with_delimiter(sep)
        .with_dtypes_slice(Some(&[DataType::UInt32, if force_string_labels {
            DataType::Utf8
        } else {
            DataType::UInt32
        }]))
        .finish()?;
    let nid = df.column("column_1")?;
    let cid = df.column("column_2")?;
    read_membership_series(g, nid, cid, mode)
}

#[pyfunction(
    name = "read_membership",
    mode = "SingletonMode::AsIs",
    sep = "'\\t'",
    force_string_labels = "false"
)]
pub fn py_read_membership_file(
    g: &Graph,
    filepath: &str,
    sep: char,
    mode: SingletonMode,
    force_string_labels: bool,
) -> PyResult<PyObject> {
    let mut df = read_membership_file(g, filepath, sep as u8, mode, force_string_labels).unwrap();
    let translated = translate_df(&mut df)?;
    Ok(translated)
}

#[pyfunction(name = "read_membership_series", mode = "SingletonMode::AsIs")]
pub fn py_from_memberships(
    g: &Graph,
    nodes: &PyAny,
    cids: &PyAny,
    mode: SingletonMode,
) -> PyResult<PyObject> {
    let nodes = ffi::py_series_to_rust_series(nodes)?;
    let cids = ffi::py_series_to_rust_series(cids)?;
    let mut df = read_membership_series(g, &nodes, &cids, mode).unwrap();
    translate_df(&mut df)
}

#[pyfunction(name = "read_json", mode = "SingletonMode::AsIs")]
pub fn py_read_json(g: &Graph, filepath: &str, mode: SingletonMode) -> PyResult<PyObject> {
    let mut df = read_json(g, filepath, mode).unwrap();
    translate_df(&mut df)
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
                    series.u32().unwrap().into_iter().flatten().for_each(|x| {
                        match g.retrieve(x as usize) {
                            Some(internal_id) => {
                                bitmap.insert(internal_id as u32);
                            }
                            None => {
                                if seen_nonexistent {
                                    panic!("Nonexistent node that is not singleton: {}", x);
                                }
                                seen_nonexistent = true;
                            }
                        }
                    });
                    bitmap.into()
                },
            )
        })
        .collect();
    Ok(sets.to_series())
}

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
        translate_df(&mut df)
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

    fn covered_edges_count(&self, n: &PyAny) -> PyResult<u64> {
        let series = ffi::py_series_to_rust_series(n)?;
        let g = &self.data;
        let edgesets = iter_roaring(&series)
            .map(|it| it.try_into().unwrap())
            .par_bridge()
            .map(|it| edgeset(g, &it))
            .collect::<Vec<_>>();
        Ok(edgesets.union().len() as u64)
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
    let out = rust_label_cc(g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "cc_size")]
pub fn py_label_cc_size(g: &Graph, series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_label_cc_size(g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}

#[pyfunction(name = "nodeset_to_list")]
pub fn py_nodeset_to_list(g: &Graph, series: &PyAny) -> PyResult<PyObject> {
    let series = ffi::py_series_to_rust_series(series)?;
    let out = rust_nodeset_to_list(g, &series).unwrap();
    ffi::rust_series_to_py_series(&out)
}


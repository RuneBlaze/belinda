use aocluster::{belinda::{
    ClusteringHandle, EnrichedGraph, GraphStats, RichCluster, RichClustering, ClusteringSource,
}, aoc::rayon};
use pyo3::{prelude::*, types::{PyDict, PyList}};
use std::{collections::HashMap, sync::{Arc, Mutex}, borrow::BorrowMut};

#[pyfunction]
pub fn set_nthreads(nthreads: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(nthreads)
        .build_global()
        .unwrap();
}

#[pyclass]
pub struct Graph {
    data: Arc<EnrichedGraph>,
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
    #[args(
        py_kwargs = "**"
    )]
    fn new(py: Python, graph: &Graph, filepath: &str, py_kwargs: Option<&PyDict>) -> PyResult<Self> {
        let mut source = ClusteringSource::Unknown;
        if let Some(kwargs) = py_kwargs {
            if let Some(cpm_resolution) = kwargs.get_item("cpm") {
                source = ClusteringSource::Cpm(cpm_resolution.extract()?);
            }
        }
        let raw_data = py.allow_threads(move || {
            let mut clus = RichClustering::<true>::pack_from_clustering(
                graph.data.clone(),
                aocluster::Clustering::parse_from_file(&graph.data.graph, filepath, false).unwrap(),
            );
            clus.source = source;
            clus
        });
        Ok(Clustering {
            data: Arc::new(raw_data),
        })
    }

    fn select_in(&self, ids: &PyList) -> PyResult<ClusteringSubset> {
        let ids: Vec<u64> = ids.extract()?;
        let data = ClusteringSubset {
            data: ClusteringHandle { graph: self.data.graph.clone(), clustering: self.data.clone(), cluster_ids: ids.into_iter().collect() },
        };
        Ok(data)
    }

    fn select(&self, f: &PyAny) -> PyResult<ClusteringSubset> {
        if !f.is_callable() {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a callable",
            ));
        } else {
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
                .map(|(k, v)| k)
                .copied()
                .collect();
            Ok(ClusteringSubset {
                data: ClusteringHandle {
                    graph: self.data.graph.clone(),
                    clustering: self.data.clone(),
                    cluster_ids: v,
                },
            })
        }
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
        self.data.percentiles.iter().cloned().collect()
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

#[pymethods]
impl ClusteringSubset {
    fn compute_statistics(&self, py: Python) -> StatsWrapper {
        py.allow_threads(move || {
            let stats = self.data.stats();
            StatsWrapper::from_graph_stats(stats)
        })
    }

    #[getter]
    fn keys(&self) -> Vec<u64> {
        self.data.cluster_ids.iter().cloned().collect()
    }

    fn compute_size_diff(&self, rhs: &Clustering) -> (u32, SummarizedDistributionWrapper) {
        let (diff, dist) = self.data.size_diff(rhs.data.as_ref());
        (diff, SummarizedDistributionWrapper::new(dist))
    }
}

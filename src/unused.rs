










// #[pyclass]
// pub struct Clustering {
//     data: Arc<RichClustering<true>>,
// }

// #[pyclass]
// pub struct ClusteringSubset {
//     data: ClusteringHandle<true>,
// }

// #[pymethods]
// impl Clustering {
//     #[new]
//     #[args(py_kwargs = "**")]
//     fn new(
//         py: Python,
//         graph: &Graph,
//         filepath: &str,
//         py_kwargs: Option<&PyDict>,
//     ) -> PyResult<Self> {
//         let mut source = ClusteringSource::Unknown;
//         if let Some(kwargs) = py_kwargs {
//             if let Some(cpm_resolution) = kwargs.get_item("cpm") {
//                 source = ClusteringSource::Cpm(cpm_resolution.extract()?);
//             }
//         }
//         let raw_data = py.allow_threads(move || {
//             let mut clus =
//                 RichClustering::<true>::pack_from_file(graph.data.clone(), filepath).unwrap();
//             clus.source = source;
//             clus
//         });
//         Ok(Clustering {
//             data: Arc::new(raw_data),
//         })
//     }

//     fn __getitem__(&self, ids: &PyList) -> PyResult<ClusteringSubset> {
//         let ids: Vec<u32> = ids.extract()?;
//         let data = ClusteringSubset {
//             data: ClusteringHandle::new(self.data.clone(), ids.into_iter().collect(), false),
//         };
//         Ok(data)
//     }

//     fn filter(&self, f: &PyAny) -> PyResult<ClusteringSubset> {
//         let v = self
//             .data
//             .clusters
//             .iter()
//             .filter(|(_k, v)| {
//                 f.call((ClusterSkeleton::from_cluster(v),), None)
//                     .unwrap()
//                     .extract()
//                     .unwrap()
//             })
//             .map(|(k, _v)| k)
//             .copied()
//             .collect();
//         let has_singletons = f
//             .call(
//                 (ClusterSkeleton {
//                     n: 1,
//                     m: 0,
//                     c: 0,
//                     mcd: 0,
//                     vol: 0,
//                 },),
//                 None,
//             )
//             .unwrap()
//             .extract()
//             .unwrap();
//         Ok(ClusteringSubset {
//             data: ClusteringHandle::new(self.data.clone(), v, has_singletons),
//         })
//     }

//     pub fn __str__(&self) -> PyResult<String> {
//         Ok(format!(
//             "Clustering(covered_nodes={}, size={})",
//             self.data.cover.len(),
//             self.data.clusters.len(),
//         ))
//     }

//     pub fn size(&self) -> usize {
//         self.data.clusters.len()
//     }
// }

// #[pyclass(name = "ClusteringStats")]
// pub struct StatsWrapper {
//     #[pyo3(get)]
//     num_clusters: u32,
//     #[pyo3(get)]
//     covered_nodes: u32,
//     #[pyo3(get)]
//     covered_edges: u64,
//     #[pyo3(get)]
//     total_nodes: u32,
//     #[pyo3(get)]
//     total_edges: u64,
//     #[pyo3(get)]
//     distributions: HashMap<String, SummarizedDistributionWrapper>,
// }

// impl StatsWrapper {
//     pub fn from_graph_stats(graph_stats: GraphStats) -> Self {
//         StatsWrapper {
//             num_clusters: graph_stats.num_clusters,
//             covered_nodes: graph_stats.covered_nodes,
//             covered_edges: graph_stats.covered_edges,
//             total_nodes: graph_stats.total_nodes,
//             total_edges: graph_stats.total_edges,
//             distributions: graph_stats
//                 .statistics
//                 .into_iter()
//                 .map(|(k, v)| {
//                     (
//                         k.to_string().to_lowercase(),
//                         SummarizedDistributionWrapper::new(v),
//                     )
//                 })
//                 .collect(),
//         }
//     }
// }

// #[pyclass(name = "SummarizedDistribution")]
// #[derive(Debug, Clone)]
// pub struct SummarizedDistributionWrapper {
//     data: aocluster::belinda::SummarizedDistribution,
// }

// impl SummarizedDistributionWrapper {
//     fn new(data: aocluster::belinda::SummarizedDistribution) -> Self {
//         SummarizedDistributionWrapper { data }
//     }
// }

// #[pymethods]
// impl SummarizedDistributionWrapper {
//     #[getter]
//     pub fn percentiles(&self) -> Vec<f64> {
//         self.data.percentiles.to_vec()
//     }

//     #[getter]
//     pub fn minimum(&self) -> f64 {
//         self.data.minimum()
//     }

//     #[getter]
//     pub fn maximum(&self) -> f64 {
//         self.data.maximum()
//     }

//     #[getter]
//     pub fn median(&self) -> f64 {
//         self.data.median()
//     }
// }

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

// #[pymethods]
// impl ClusteringSubset {
//     fn compute_statistics(&self, py: Python) -> StatsWrapper {
//         py.allow_threads(move || {
//             let stats = self.data.stats();
//             StatsWrapper::from_graph_stats(stats)
//         })
//     }

//     fn __getitem__(&self, key: u32) -> PyResult<ClusterSkeleton> {
//         let clus = &self.data.clustering;
//         if let Some(cluster) = clus.clusters.get(&key) {
//             Ok(ClusterSkeleton::from_cluster(cluster))
//         } else {
//             Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>(
//                 "Cluster not found",
//             ))
//         }
//     }

//     fn to_df(&self) -> PyResult<PyObject> {
//         let in_scope_clusters = self
//             .data
//             .cluster_ids
//             .iter()
//             .map(|it| &self.data.clustering.clusters[&it])
//             .collect::<Vec<_>>();
//         let mcd = in_scope_clusters
//             .iter()
//             .map(|it| it.mcd)
//             .collect::<Vec<_>>();
//         let mut df = df!("mcd" => mcd).unwrap();
//         translate_df(&mut df)
//     }

//     fn keys(&self) -> Vec<u32> {
//         self.data.cluster_ids.iter().collect()
//     }

//     fn size(&self) -> u64 {
//         self.data.cluster_ids.len()
//     }

//     fn compute_size_diff(&self, rhs: &Clustering) -> (u32, SummarizedDistributionWrapper) {
//         let (diff, dist) = self.data.size_diff(rhs.data.as_ref());
//         (diff, SummarizedDistributionWrapper::new(dist))
//     }

//     #[getter]
//     fn cluster_sizes(&self) -> Vec<u32> {
//         let d = &self.data;
//         d.cluster_ids
//             .iter()
//             .map(|k| d.clustering.clusters.get(&k).unwrap().nodes.len() as u32)
//             .collect()
//     }

//     #[getter]
//     fn node_coverage(&self) -> f64 {
//         self.data.get_covered_nodes() as f64 / self.data.graph.graph.n() as f64
//     }

//     #[getter]
//     fn num_singletons(&self) -> u32 {
//         if self.data.has_singletons {
//             self.data.clustering.singleton_clusters.len() as u32
//         } else {
//             0
//         }
//     }

//     fn node_multiplicities(&self) -> Vec<u32> {
//         let raw_mult = &self.data.node_multiplicity;
//         let mut mults: Vec<_> = self
//             .data
//             .covered_nodes
//             .iter()
//             .map(|n| raw_mult[n as usize])
//             .collect();
//         if self.data.has_singletons {
//             mults.extend((0..self.num_singletons()).map(|_| 1));
//         }
//         mults
//     }

//     #[getter]
//     fn node_multiplicities_dist(&self) -> SummarizedDistributionWrapper {
//         SummarizedDistributionWrapper::new(
//             self.node_multiplicities()
//                 .into_iter()
//                 .map(|it| it as f64)
//                 .collect(),
//         )
//     }

//     fn __str__(&self) -> PyResult<String> {
//         Ok(format!(
//             "ClusteringSubset(size={}, node_coverage={:.1}%, is_overlapping={})",
//             self.data.cluster_ids.len(),
//             self.node_coverage() * 100.0,
//             self.data.is_overlapping
//         ))
//     }
// }


// #[pyclass]
// pub struct ClusterSkeleton {
//     #[pyo3(get)]
//     n: u64,
//     #[pyo3(get)]
//     m: u64,
//     #[pyo3(get)]
//     c: u64,
//     #[pyo3(get)]
//     mcd: u64,
//     #[pyo3(get)]
//     vol: u64,
// }

// #[pymethods]
// impl ClusterSkeleton {
//     pub fn __str__(&self) -> PyResult<String> {
//         Ok(format!(
//             "ClusterSkeleton(n={}, m={}, c={})",
//             self.n, self.m, self.c,
//         ))
//     }
// }

// impl From<RichCluster> for ClusterSkeleton {
//     fn from(cluster: RichCluster) -> Self {
//         ClusterSkeleton {
//             n: cluster.n,
//             m: cluster.m,
//             c: cluster.c,
//             mcd: cluster.mcd,
//             vol: cluster.vol,
//         }
//     }
// }

// impl ClusterSkeleton {
//     fn from_cluster(cluster: &RichCluster) -> Self {
//         ClusterSkeleton {
//             n: cluster.n,
//             m: cluster.m,
//             c: cluster.c,
//             mcd: cluster.mcd,
//             vol: cluster.vol,
//         }
//     }
// }
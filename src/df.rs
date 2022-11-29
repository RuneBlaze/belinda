use polars::export::arrow::array::{Array, BinaryArray, MutableBinaryArray};
use polars::prelude::PolarsError;
use polars::prelude::*;
use polars::{prelude::NamedFrom, series::Series};
use roaring::{MultiOps, RoaringBitmap, RoaringTreemap};

pub type ArrayRef = Box<dyn Array>;

pub enum EfficientSet {
    SmallSet(RoaringBitmap),
    BigSet(RoaringTreemap),
}

impl EfficientSet {
    pub fn len(&self) -> u64 {
        match self {
            EfficientSet::SmallSet(set) => set.len() as u64,
            EfficientSet::BigSet(set) => set.len() as u64,
        }
    }
}

pub trait VecEfficientSet {
    fn union(self) -> EfficientSet;
    fn to_series(self) -> Series;
}

impl VecEfficientSet for Vec<EfficientSet> {
    fn union(self) -> EfficientSet {
        let _smallset: Vec<EfficientSet> = vec![];
        let _bigset: Vec<EfficientSet> = vec![];
        let (smallset, bigset): (Vec<_>, Vec<_>) = self.into_iter().partition(|s| match s {
            EfficientSet::SmallSet(_) => true,
            EfficientSet::BigSet(_) => false,
        });
        if !smallset.is_empty() {
            EfficientSet::SmallSet(
                smallset
                    .into_iter()
                    .map(|it| it.try_into().unwrap())
                    .collect::<Vec<RoaringBitmap>>()
                    .union(),
            )
        } else {
            EfficientSet::BigSet(
                bigset
                    .into_iter()
                    .map(|it| it.try_into().unwrap())
                    .collect::<Vec<RoaringTreemap>>()
                    .union(),
            )
        }
    }

    fn to_series(self) -> Series {
        build_series_from_sets(self)
    }
}

impl TryFrom<EfficientSet> for RoaringBitmap {
    type Error = PolarsError;

    fn try_from(value: EfficientSet) -> Result<Self, Self::Error> {
        match value {
            EfficientSet::SmallSet(set) => Ok(set),
            EfficientSet::BigSet(_set) => Err(PolarsError::InvalidOperation(
                "Cannot convert BigSet to RoaringBitmap".into(),
            )),
        }
    }
}

impl From<RoaringBitmap> for EfficientSet {
    fn from(set: RoaringBitmap) -> Self {
        EfficientSet::SmallSet(set)
    }
}

impl TryFrom<EfficientSet> for RoaringTreemap {
    type Error = PolarsError;

    fn try_from(value: EfficientSet) -> Result<Self, Self::Error> {
        match value {
            EfficientSet::SmallSet(_set) => Err(PolarsError::InvalidOperation(
                "Cannot convert SmallSet to RoaringTreemap".into(),
            )),
            EfficientSet::BigSet(set) => Ok(set),
        }
    }
}

impl From<RoaringTreemap> for EfficientSet {
    fn from(set: RoaringTreemap) -> Self {
        EfficientSet::BigSet(set)
    }
}

pub fn serialize_set<W>(set: &EfficientSet, mut writer: W) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    match set {
        EfficientSet::SmallSet(bm) => {
            writer.write_all(&[0u8])?;
            bm.serialize_into(writer)?;
        }
        EfficientSet::BigSet(tm) => {
            writer.write_all(&[1u8])?;
            tm.serialize_into(writer)?;
        }
    }
    Ok(())
}

pub fn deserialize_set<R>(mut reader: R) -> anyhow::Result<EfficientSet>
where
    R: std::io::Read,
{
    let mut buf = [0u8];
    reader.read_exact(&mut buf)?;
    match buf[0] {
        0 => {
            let bm = RoaringBitmap::deserialize_from(reader)?;
            Ok(EfficientSet::SmallSet(bm))
        }
        1 => {
            let tm = RoaringTreemap::deserialize_from(reader)?;
            Ok(EfficientSet::BigSet(tm))
        }
        _ => Err(anyhow::anyhow!("Invalid EfficientSet serialization")),
    }
}

pub fn build_series_from_bitmap(nodesets: Vec<RoaringBitmap>) -> Series {
    let mut arr = MutableBinaryArray::<i32>::new();
    for n in nodesets {
        let mut bytes = vec![];
        serialize_set(&EfficientSet::SmallSet(n), &mut bytes).unwrap();
        arr.push(Some(bytes))
    }
    let result: BinaryArray<i32> = arr.into();
    Series::try_from(("nodes", Box::new(result) as ArrayRef)).unwrap()
}

pub fn build_series_from_treemap(nodesets: Vec<RoaringTreemap>) -> Series {
    let mut arr = MutableBinaryArray::<i32>::new();
    for n in nodesets {
        let mut bytes = vec![];
        serialize_set(&EfficientSet::BigSet(n), &mut bytes).unwrap();
        arr.push(Some(bytes))
    }
    let result: BinaryArray<i32> = arr.into();
    Series::try_from(("nodes", Box::new(result) as ArrayRef)).unwrap()
}

pub fn build_series_from_sets(nodesets: Vec<EfficientSet>) -> Series {
    let mut arr = MutableBinaryArray::<i32>::new();
    for n in nodesets {
        let mut bytes = vec![];
        serialize_set(&n, &mut bytes).unwrap();
        arr.push(Some(bytes))
    }
    let result: BinaryArray<i32> = arr.into();
    Series::try_from(("nodes", Box::new(result) as ArrayRef)).unwrap()
}

pub(crate) fn iter_roaring(series: &Series) -> impl Iterator<Item = EfficientSet> + '_ {
    let chunks = series.binary().expect("series was not a list type");
    let iter = chunks.into_iter();
    iter.map(|row| {
        let value = row.expect("row is null");
        let mut reader = std::io::Cursor::new(value);
        deserialize_set(&mut reader).unwrap()
    })
}

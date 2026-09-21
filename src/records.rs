use crate::error::RecordConversionError;
use crate::models::common::{atk_param_pc::AtkParamPc, bullet::Bullet, magic::Magic};

#[derive(Debug, Clone, Copy)]
pub enum RecordKind {
    AtkParamPc,
    Bullet,
    Magic,
}

#[derive(Debug)]
pub enum Param {
    AtkParamPC,
    Bullet,
    Magic,
}

#[derive(Debug)]
pub enum Record {
    AtkParamPC(AtkParamPc),
    Bullet(Bullet),
    Magic(Magic),
}

#[derive(Debug)]
pub enum SearchField {
    Name(String),
    ID(i64),
}

impl Record {
    pub fn kind(&self) -> RecordKind {
        match self {
            Self::AtkParamPC(_) => RecordKind::AtkParamPc,
            Self::Bullet(_) => RecordKind::Bullet,
            Self::Magic(_) => RecordKind::Magic,
        }
    }
}

impl TryFrom<Record> for Magic {
    type Error = RecordConversionError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        match record {
            Record::Magic(magic) => Ok(magic),
            other => Err(RecordConversionError::ExpectedMagic(other.kind())),
        }
    }
}

impl TryFrom<Record> for Bullet {
    type Error = RecordConversionError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        match record {
            Record::Bullet(bullet) => Ok(bullet),
            other => Err(RecordConversionError::ExpectedBullet(other.kind())),
        }
    }
}

impl TryFrom<Record> for AtkParamPc {
    type Error = RecordConversionError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        match record {
            Record::AtkParamPC(atk_param_pc) => Ok(atk_param_pc),
            other => Err(RecordConversionError::ExpectedAtkParamPc(other.kind())),
        }
    }
}

impl SearchField {
    pub fn description(&self) -> String {
        match self {
            Self::Name(name) => format!("name = {name}"),
            Self::ID(id) => format!("id = {id}"),
        }
    }
}

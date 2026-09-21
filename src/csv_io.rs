use crate::error::AppError;
use crate::models::common::{atk_param_pc::AtkParamPc, bullet::Bullet, magic::Magic};
use crate::records::{Param, Record, SearchField};
use crate::types::traits::Searchable;

fn load_record<T>(file_name: &str, search_field: &SearchField) -> Result<Option<T>, csv::Error>
where
    T: for<'de> serde::Deserialize<'de> + Searchable,
{
    let path = format!("./csv/common/{file_name}.csv");
    let mut reader = csv::Reader::from_path(path)?;

    for result in reader.deserialize::<T>() {
        let record = result?;

        if record.matches(search_field) {
            return Ok(Some(record));
        }
    }

    Ok(None)
}

fn load_records<T>(file_name: &str, search_field: &SearchField) -> Result<Vec<T>, csv::Error>
where
    T: for<'de> serde::Deserialize<'de> + Searchable,
{
    let path = format!("./csv/common/{file_name}.csv");
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.deserialize::<T>() {
        let record = result?;

        if record.matches(search_field) {
            records.push(record);
        }
    }

    Ok(records)
}

fn find_record(param: Param, search_field: &SearchField) -> Result<Option<Record>, csv::Error> {
    match param {
        Param::AtkParamPC => load_record::<AtkParamPc>("AtkParam_Pc", search_field)
            .map(|record| record.map(Record::AtkParamPC)),

        Param::Bullet => {
            load_record::<Bullet>("Bullet", search_field).map(|record| record.map(Record::Bullet))
        }

        Param::Magic => {
            load_record::<Magic>("Magic", search_field).map(|record| record.map(Record::Magic))
        }
    }
}

pub fn find_atk_param_pc(search_field: &SearchField) -> Result<AtkParamPc, AppError> {
    load_record::<AtkParamPc>("AtkParam_Pc", search_field)?.ok_or_else(|| AppError::NotFound {
        record_type: "AtkParam_Pc",
        field: search_field.description(),
    })
}

pub fn find_bullet(search_field: &SearchField) -> Result<Bullet, AppError> {
    load_record::<Bullet>("Bullet", search_field)?.ok_or_else(|| AppError::NotFound {
        record_type: "Bullet",
        field: search_field.description(),
    })
}

pub fn find_magic(search_field: &SearchField) -> Result<Magic, AppError> {
    load_record::<Magic>("Magic", search_field)?.ok_or_else(|| AppError::NotFound {
        record_type: "Magic",
        field: search_field.description(),
    })
}

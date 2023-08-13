use std::arch::x86_64::__cpuid;
use chrono::NaiveDate;
use crate::finanzapi::{FinanzData, ValueInformation};
use itertools::{Itertools, min, MinMaxResult};


pub fn parse_to_view_data(finanz_data: FinanzData) -> Result<ViewData, ParseError> {
    let mut data = ViewData {
        key: (&finanz_data.mata_data.symbol).clone(),
        from: Default::default(),
        to: Default::default(),
        labels: vec![],
        range: vec![],
        start_end: vec![],
        colors: vec![],
    };
    let minmax:MinMaxResult<(&String, &ValueInformation)> = (&finanz_data.weekly_time_series).into_iter()
        .minmax_by(|(label_a, _), (label_b, _)| NaiveDate::parse_from_str(label_a.as_str(), "%Y-%m-%d").expect("Label a date").cmp(&NaiveDate::parse_from_str(label_b.as_str(), "%Y-%m-%d").expect("Label b date")));
    match minmax {
        MinMaxResult::NoElements => { panic!("Keine MinMax")}
        MinMaxResult::OneElement(_) => {panic!("Nur Min")}
        MinMaxResult::MinMax((min, _), (max,_)) => {
            data.from =  NaiveDate::parse_from_str(min.as_str(), "%Y-%m-%d")?;
            data.to =  NaiveDate::parse_from_str(max.as_str(), "%Y-%m-%d")?;
        }
    }
    let min_max_volume = (&finanz_data.weekly_time_series).into_iter()
        .minmax_by(|(_, a), (_, b)| a.volume.total_cmp(&b.volume));
    let min_volume:f32;
    let max_volume:f32;
    match min_max_volume {
        MinMaxResult::NoElements => {min_volume = 0.0; max_volume = 0.0;}
        MinMaxResult::OneElement((_, value)) => {min_volume = 0.0; max_volume = value.volume.clone();}
        MinMaxResult::MinMax((_,a), (_,b)) => {min_volume = a.volume.clone(); max_volume = b.volume.clone();}
    }

    finanz_data.weekly_time_series.into_iter()
        .sorted_by(|(label_a, _), (label_b,_)| NaiveDate::parse_from_str(label_a.as_str(),"%Y-%m-%d").expect("Label a date").cmp(&NaiveDate::parse_from_str(label_b.as_str(), "%Y-%m-%d").expect("Label b date")))
        .for_each(|(label, value_information)| {
            data.labels.push(label.clone());
            data.range.push([value_information.high, value_information.low]);
            data.start_end.push([value_information.open, value_information.close]);
            data.colors.push(match ( value_information.volume - min_volume)/(max_volume - min_volume) {
                x if x > 0.9 => "rgba(255, 99, 132, 0.2)".to_string(),
                x if x > 0.75 => "rgba(255, 159, 64, 0.2)".to_string(),
                x if x >= 0.5 => "rgba(255, 205, 86, 0.2)".to_string(),
                _ => "rgba(75, 192, 192, 0.2)".to_string(),
            })
        });

    
    
    Ok(data)
}



#[derive(Debug)]
pub struct ViewData {
    pub key: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub labels: Vec<String>,
    pub range: Vec<[f32; 2]>,
    pub start_end: Vec<[f32; 2]>,
    pub colors: Vec<String>,
}

#[derive(Debug)]
pub enum ParseError {
    ChronoParse(chrono::ParseError),
}

impl From<chrono::ParseError> for ParseError {
    fn from(value: chrono::ParseError) -> Self {
        ParseError::ChronoParse(value)
    }
}



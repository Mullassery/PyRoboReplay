pub mod adapters;
pub mod core;
pub mod cli;

use pyo3::prelude::*;

#[pyfunction]
fn create_mission(name: String) -> PyResult<String> {
    let mission = core::MissionRecord::new(name);
    Ok(mission.id.to_string())
}

#[pyfunction]
fn get_mission_info(mission_id: String) -> PyResult<String> {
    Ok(format!("Mission: {}", mission_id))
}

#[pymodule]
fn pyroboreplay(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_mission, m)?)?;
    m.add_function(wrap_pyfunction!(get_mission_info, m)?)?;
    Ok(())
}

use rust_xlsxwriter::{Format, Workbook, XlsxError};
use std::path::Path;

use crate::db::{SessionRow, SummaryRow};

fn level_it(s: &str) -> &str {
    match s {
        "video" => "Con video",
        "highlighted" => "Evidenziato",
        "picture_only" => "Solo immagini",
        "independent" => "Da solo",
        other => other,
    }
}

pub fn write_xlsx(
    path: &Path,
    sessions: &[SessionRow],
    summary: &[SummaryRow],
) -> Result<(), XlsxError> {
    let mut wb = Workbook::new();
    let head = Format::new().set_bold();
    let date_fmt = Format::new().set_num_format("dd/mm/yyyy hh:mm");
    let one_dec = Format::new().set_num_format("0.0");

    // лист 1: итоги по каждому человеку
    let ws = wb.add_worksheet();
    ws.set_name("Riepilogo")?;
    let titles = [
        "Persona", "Giri", "Errori (media)", "Tempo medio (s)",
        "Miglior tempo (s)", "Ultimo giro", "Livello attuale",
    ];
    for (c, t) in titles.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *t, &head)?;
    }
    for (i, s) in summary.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write_string(r, 0, &s.player)?;
        ws.write_number(r, 1, s.laps as f64)?;
        ws.write_number_with_format(r, 2, s.avg_errors, &one_dec)?;
        ws.write_number_with_format(r, 3, s.avg_seconds, &one_dec)?;
        ws.write_number(r, 4, s.best_seconds as f64)?;
        ws.write_number_with_format(r, 5, s.last_date, &date_fmt)?;
        ws.write_string(r, 6, level_it(&s.level))?;
    }
    ws.set_column_width(0, 18)?;
    ws.set_column_width(2, 16)?;
    ws.set_column_width(3, 16)?;
    ws.set_column_width(4, 18)?;
    ws.set_column_width(5, 18)?;
    ws.set_column_width(6, 16)?;
    ws.set_freeze_panes(1, 0)?;

    // лист 2: каждый круг отдельной строкой
    let ws = wb.add_worksheet();
    ws.set_name("Giri")?;
    let titles = ["Data", "Persona", "Livello", "Errori", "Tempo (s)"];
    for (c, t) in titles.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *t, &head)?;
    }
    for (i, s) in sessions.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write_number_with_format(r, 0, s.date, &date_fmt)?;
        ws.write_string(r, 1, &s.player)?;
        ws.write_string(r, 2, level_it(&s.level))?;
        ws.write_number(r, 3, s.errors as f64)?;
        ws.write_number(r, 4, s.seconds as f64)?;
    }
    ws.set_column_width(0, 18)?;
    ws.set_column_width(1, 18)?;
    ws.set_column_width(2, 16)?;
    ws.set_freeze_panes(1, 0)?;
    ws.autofilter(0, 0, sessions.len() as u32, 4)?;

    wb.save(path)
}

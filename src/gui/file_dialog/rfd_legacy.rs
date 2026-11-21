//! Alternativa legada para portal/RFD e plataformas sem Zenity.

use super::{Picked, Request};

/// O `rfd` devolve `None` tanto para cancelamento quanto para indisponibilidade.
/// Essa limitação residual existe somente no backend legado: representamos o
/// resultado como cancelamento para não inventar uma falha, e mantemos os
/// caminhos manuais visíveis. O backend Zenity, usado no caminho Linux de
/// referência, diferencia cancelamento, indisponibilidade e falha.
pub fn run(request: Request) -> Picked {
    let result = std::panic::catch_unwind(|| match request {
        Request::OpenFile { title } => rfd::FileDialog::new().set_title(title).pick_file(),
        Request::SaveFile {
            title,
            file_name,
            filter_name,
            extensions,
        } => {
            let extensions: Vec<&str> = extensions.iter().map(String::as_str).collect();
            rfd::FileDialog::new()
                .set_title(title)
                .set_file_name(file_name)
                .add_filter(filter_name, &extensions)
                .save_file()
        }
        Request::Folder { title } => rfd::FileDialog::new().set_title(title).pick_folder(),
    });
    match result {
        Ok(Some(path)) => Picked::Path(path),
        Ok(None) => Picked::Cancelled,
        Err(_) => Picked::Failed(
            "o backend RFD falhou; informe o caminho manualmente ou selecione outro backend".into(),
        ),
    }
}

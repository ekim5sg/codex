use js_sys::{Array, Promise, Uint8Array};
use std::collections::HashSet;
use std::io::{Cursor, Write};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, Blob, Document, Event, File, HtmlAnchorElement, HtmlButtonElement, HtmlElement,
    HtmlInputElement, Url,
};
use zip::write::SimpleFileOptions;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    setup_ui()?;
    Ok(())
}

fn setup_ui() -> Result<(), JsValue> {
    let document = document()?;

    let app = get_html_element(&document, "app")?;
    app.set_inner_html(
        r#"
            <h1>Mobile Directory Zipper</h1>
            <p>
                Choose a folder and create a clean ZIP archive suitable for sharing from mobile browsers.
            </p>
            <label class="picker">
                <span>Folder</span>
                <input id="folder-input" type="file" webkitdirectory directory multiple />
            </label>
            <label class="filename">
                <span>ZIP file name</span>
                <input id="zip-name" type="text" value="archive.zip" />
            </label>
            <button id="zip-btn" type="button">Create ZIP</button>
            <p id="status" class="status">Select a folder to get started.</p>
        "#,
    );

    let zip_button: HtmlButtonElement = get_element_cast(&document, "zip-btn")?;

    let closure = Closure::wrap(Box::new(move |_event: Event| {
        wasm_bindgen_futures::spawn_local(async {
            if let Err(err) = create_zip_from_selection().await {
                let _ = set_status(&format!("Failed: {}", js_error_string(&err)));
            }
        });
    }) as Box<dyn FnMut(_)>);

    zip_button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    Ok(())
}

async fn create_zip_from_selection() -> Result<(), JsValue> {
    set_status("Reading folder…")?;

    let document = document()?;
    let input: HtmlInputElement = get_element_cast(&document, "folder-input")?;
    let zip_name_input: HtmlInputElement = get_element_cast(&document, "zip-name")?;
    let zip_name = sanitize_zip_name(&zip_name_input.value());

    let files = input
        .files()
        .ok_or_else(|| JsValue::from_str("File picker is not available"))?;

    if files.length() == 0 {
        set_status("No files selected. Pick a folder first.")?;
        return Ok(());
    }

    let mut entries = Vec::new();
    for idx in 0..files.length() {
        let file = files
            .get(idx)
            .ok_or_else(|| JsValue::from_str("Could not read selected file"))?;
        let raw_path = file.webkit_relative_path();

        let path = if raw_path.is_empty() {
            file.name()
        } else {
            raw_path
        };

        entries.push((path, file));
    }

    let clean_entries = normalize_entries(entries);

    let mut zip_buffer = Cursor::new(Vec::<u8>::new());
    {
        let mut zip_writer = zip::ZipWriter::new(&mut zip_buffer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (path, file) in clean_entries {
            let bytes = read_file_bytes(&file).await?;
            zip_writer
                .start_file(path, options)
                .map_err(|e| JsValue::from_str(&format!("Could not add file to ZIP: {e}")))?;
            zip_writer
                .write_all(&bytes)
                .map_err(|e| JsValue::from_str(&format!("Could not write ZIP content: {e}")))?;
        }

        zip_writer
            .finish()
            .map_err(|e| JsValue::from_str(&format!("Could not finalize ZIP: {e}")))?;
    }

    download_zip(zip_buffer.into_inner(), &zip_name)?;
    set_status(&format!("ZIP ready: {zip_name}"))?;

    Ok(())
}

fn normalize_entries(entries: Vec<(String, File)>) -> Vec<(String, File)> {
    let normalized_paths: Vec<String> = entries
        .iter()
        .map(|(path, _)| normalize_path(path))
        .collect();

    let common_prefix = common_root_folder(&normalized_paths);

    entries
        .into_iter()
        .zip(normalized_paths)
        .map(|((_, file), path)| {
            let path = strip_common_prefix(&path, common_prefix.as_deref());
            (path, file)
        })
        .collect()
}

fn normalize_path(raw: &str) -> String {
    raw.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

fn common_root_folder(paths: &[String]) -> Option<String> {
    let mut first_segments = HashSet::new();

    for path in paths {
        let mut parts = path.split('/');
        match parts.next() {
            Some(first) if !first.is_empty() => {
                first_segments.insert(first.to_string());
            }
            _ => return None,
        }
    }

    if first_segments.len() == 1 {
        first_segments.into_iter().next()
    } else {
        None
    }
}

fn strip_common_prefix(path: &str, common_prefix: Option<&str>) -> String {
    if let Some(prefix) = common_prefix {
        let with_sep = format!("{prefix}/");
        if path.starts_with(&with_sep) {
            return path.trim_start_matches(&with_sep).to_string();
        }
    }
    path.to_string()
}

async fn read_file_bytes(file: &File) -> Result<Vec<u8>, JsValue> {
    let promise: Promise = file.array_buffer();
    let js_buffer = JsFuture::from(promise).await?;
    let bytes = Uint8Array::new(&js_buffer).to_vec();
    Ok(bytes)
}

fn download_zip(bytes: Vec<u8>, file_name: &str) -> Result<(), JsValue> {
    let array = Uint8Array::from(bytes.as_slice());
    let chunks = Array::new();
    chunks.push(&array.buffer());

    let blob = Blob::new_with_u8_array_sequence(&chunks)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    let document = document()?;
    let anchor: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;

    anchor.set_href(&url);
    anchor.set_download(file_name);
    anchor.style().set_property("display", "none")?;

    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("Document body is unavailable"))?;

    body.append_child(&anchor)?;
    anchor.click();
    body.remove_child(&anchor)?;

    Url::revoke_object_url(&url)?;
    Ok(())
}

fn sanitize_zip_name(value: &str) -> String {
    let cleaned = value.trim().replace('/', "_");
    if cleaned.is_empty() {
        "archive.zip".to_string()
    } else if cleaned.ends_with(".zip") {
        cleaned
    } else {
        format!("{cleaned}.zip")
    }
}

fn set_status(message: &str) -> Result<(), JsValue> {
    let document = document()?;
    let status = get_html_element(&document, "status")?;
    status.set_inner_text(message);
    Ok(())
}

fn document() -> Result<Document, JsValue> {
    window()
        .ok_or_else(|| JsValue::from_str("No browser window available"))?
        .document()
        .ok_or_else(|| JsValue::from_str("No document available"))
}

fn get_element_cast<T: JsCast>(document: &Document, id: &str) -> Result<T, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Missing element with id '{id}'")))?
        .dyn_into::<T>()
        .map_err(|_| JsValue::from_str(&format!("Element '{id}' had unexpected type")))
}

fn get_html_element(document: &Document, id: &str) -> Result<HtmlElement, JsValue> {
    get_element_cast::<HtmlElement>(document, id)
}

fn js_error_string(err: &JsValue) -> String {
    err.as_string()
        .unwrap_or_else(|| "Unknown JS/WASM error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn path_normalization_removes_empty_segments() {
        assert_eq!(
            normalize_path("my-folder//nested/./a.txt"),
            "my-folder/nested/a.txt"
        );
    }

    #[test]
    fn strip_common_prefix_when_same_root() {
        let root = common_root_folder(&[
            "photos/2025/img1.jpg".to_string(),
            "photos/2025/img2.jpg".to_string(),
        ]);

        assert_eq!(root.as_deref(), Some("photos"));
        assert_eq!(
            strip_common_prefix("photos/2025/img1.jpg", root.as_deref()),
            "2025/img1.jpg"
        );
    }

    #[test]
    fn keep_paths_when_multiple_roots() {
        let root = common_root_folder(&["a/file1.txt".to_string(), "b/file2.txt".to_string()]);
        assert_eq!(root, None);
    }
}

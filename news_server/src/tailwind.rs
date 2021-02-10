use duct::cmd;

pub async fn process_tailwind() -> std::io::Result<String> {
    // CSS MODULES ----
    let mut css_container = String::new();
    let modules_dir = "news_templates/modules/";

    for entry in std::fs::read_dir(modules_dir)? {
        let entry = entry?;
        let path = format!("{}/tpl.scss", entry.path().as_os_str().to_str().unwrap());
        if !std::path::Path::new(&path).exists() {
            continue;
        }

        let css = std::fs::read_to_string(path)?;

        css_container = format!("{}\n{}", css_container, css);
    }

    // CSS SVELTE ----

    let mut css_svelte_container = String::new();
    let css_svelte_dir = "news_templates/css_svelte/";

    for entry in std::fs::read_dir(css_svelte_dir)? {
        let entry = entry?.path();
        let entry_path = entry.as_os_str().to_str().unwrap();
        let css = std::fs::read_to_string(entry_path)?;
        css_svelte_container = format!("{}\n{}", css_svelte_container, css);
    }

    // COMBINE CSS ----

    let main_css = std::fs::read_to_string("news_templates/css/main.scss")?;
    let all_css = format!(
        "{}\n{}\n/*! purgecss start ignore */\n{}\n/*! purgecss end ignore */",
        main_css, css_container, css_svelte_container
    );

    std::fs::write("news_templates/css/main.css", all_css.to_owned())?;

    cmd!("postcss", "news_templates/css/main.css", "--replace").read()
}

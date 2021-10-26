function toggle_file_text_button() {
    var setting = document.documentElement.style.getPropertyValue("--file-text-opacity");
    if (setting == 0) {
        document.documentElement.style.setProperty("--file-text-opacity", 1);
    } else {
        document.documentElement.style.setProperty("--file-text-opacity", 0);
    }
}

function toggle_references_button() {
    var setting = document.documentElement.style.getPropertyValue("--reference-opacity");
    if (setting == 0) {
        document.documentElement.style.setProperty("--reference-opacity", 1);
    } else {
        document.documentElement.style.setProperty("--reference-opacity", 0);
    }
}

function reset_view_button() {
    document.getElementById("view-root").style.setProperty("transform", "none")
    document.documentElement.style.setProperty("--text-scaling", 1);
}

function load() {
    Array.from(document.getElementsByClassName("folder")).forEach(f => f.addEventListener("click", folder_click))
}

function folder_click(e) {
    document.getElementById("view-root").style.setProperty("transform", e.currentTarget.dataset.transform);
    document.documentElement.style.setProperty("--text-scaling", e.currentTarget.dataset.textScale);
    e.stopPropagation()
}
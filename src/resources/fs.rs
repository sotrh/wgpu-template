pub async fn save_text(path: &str, contents: &str) -> anyhow::Result<()> {
    Ok(async_fs::write(path, contents).await?)
}

pub async fn save_json(path: &str, contents: impl serde::Serialize) -> anyhow::Result<()> {
    let text = serde_json::to_string_pretty(&contents)?;
    save_text(path, &text).await
}

pub async fn load_text(path: &str) -> anyhow::Result<String> {
    Ok(async_fs::read_to_string(path).await?)
}

pub async fn load_json<T>(path: &str) -> anyhow::Result<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    let text = load_text(path).await?;
    let data = serde_json::from_str(&text)?;
    Ok(data)
}

pub async fn load_binary(path: &str) -> anyhow::Result<Vec<u8>> {
    Ok(async_fs::read(path).await?)
}
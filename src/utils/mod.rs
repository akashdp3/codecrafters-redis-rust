use anyhow::Context;

pub(crate) fn next_arg(
    args: &mut impl Iterator<Item = String>,
    cmd: &str,
    key: &str,
) -> anyhow::Result<String> {
    let error_msg = format!("Missing argument '{key}' for {cmd} command ");

    args.next().with_context(|| error_msg)
}

pub(crate) fn parse_stream_id(id: &str) -> anyhow::Result<(u128, u128)> {
    let (ms_time, seq_num) = id.split_once('-').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid stream id format. It should be in format '<millisecond_time>-<sequence_number>'"
        )
    })?;

    let ms_time: u128 = ms_time.parse()?;
    let seq_num: u128 = seq_num.parse()?;

    Ok((ms_time, seq_num))
}

use anyhow::Context;

pub(crate) fn next_arg(
    args: &mut impl Iterator<Item = String>,
    cmd: &str,
    key: &str,
) -> anyhow::Result<String> {
    let error_msg = format!("Missing argument '{key}' for {cmd} command ");

    args.next().with_context(|| error_msg)
}

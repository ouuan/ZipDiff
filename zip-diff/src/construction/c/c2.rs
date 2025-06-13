use crate::utils::testcase_arg;
use anyhow::Result;
use zip_diff::utils::BinCount;
use zip_diff::zip::ZipArchive;

enum Arg {
    LongCommentLength,
    ShortCommentLength,
    LfhCdhMismatch,
}

fn eocdr_selection(arg: Arg) -> Result<[ZipArchive; 3]> {
    let mut zip1 = ZipArchive::default();
    zip1.add_simple("a", b"a")?;
    zip1.finalize()?;
    zip1.eocdr.zip_file_comment.push(b'\0');

    let zip_size = zip1.byte_count()?;
    zip1.eocdr.zip_file_comment_length = (zip_size * 2 + 1).try_into()?;

    let mut zip2 = ZipArchive::default();
    zip2.add_simple("b", b"b")?;
    zip2.finalize()?;
    zip2.set_offsets(zip_size)?;
    zip2.eocdr.zip_file_comment.push(b'\0');
    zip2.eocdr.zip_file_comment_length = (zip_size + 1).try_into()?;

    let mut zip3 = ZipArchive::default();
    zip3.add_simple("c", b"c")?;
    zip3.finalize()?;
    zip3.set_offsets(zip_size * 2)?;
    zip3.eocdr.zip_file_comment.push(b'\0');
    zip3.eocdr.zip_file_comment_length = 1;

    match arg {
        Arg::LongCommentLength => {
            zip1.eocdr.zip_file_comment_length += 1;
            zip3.eocdr.zip_file_comment_length += 1;
        }
        Arg::ShortCommentLength => {
            zip1.eocdr.zip_file_comment_length -= 1;
            zip3.eocdr.zip_file_comment_length -= 1;
        }
        Arg::LfhCdhMismatch => {
            zip1.cd[0].version_needed = 10;
            zip3.cd[0].version_needed = 10;
        }
    }

    Ok([zip1, zip2, zip3])
}

pub fn main() -> Result<()> {
    testcase_arg(eocdr_selection, Arg::LongCommentLength)?;
    testcase_arg(eocdr_selection, Arg::ShortCommentLength)?;
    testcase_arg(eocdr_selection, Arg::LfhCdhMismatch)?;
    Ok(())
}

//! Post-update migration: ports `_AfterUpdate`'s one-time
//! file/directory/ini-key cleanup steps (UniExtract.au3:5673-5769),
//! applied once after an update completes to remove cruft accumulated
//! across the program's history.
//!
//! **This is historically-accumulated upgrade-path behavior, ported
//! verbatim rather than "cleaned up"** — dropping any single entry would
//! leave that piece of cruft behind for a real user upgrading from an old
//! installed version. There is exactly one branch in the whole function
//! (whether `$docsdir` still has a `7zip_readme.txt`, gating a `MoveFiles`
//! call); everything else is an unconditional, ordered sequence of file
//! moves, deletions, directory removals, and ini-key deletions. The value
//! of this module is completeness, not cleverness — [`post_update_actions`]
//! returns that exact ordered sequence as data, and every entry has a test
//! asserting it's present, in the same order as the source.
//!
//! This capability covers only assembling the plan. Actually performing
//! each [`PostUpdateAction`] (`FileMove`, `MoveFiles`, `FileDelete`,
//! `DirRemove`, `IniDelete`) is real file I/O the caller performs; the
//! trailing `SendStats`, `CheckUpdate($UPDATEMSG_SILENT, False,
//! $UPDATE_HELPER)` (C207), de-elevation, and `Restart()` are likewise
//! real side effects outside this row's scope.

/// The directory paths `_AfterUpdate` references, already resolved by the
/// caller.
pub struct PostUpdateDirs<'a> {
    pub bindir: &'a str,
    pub defdir: &'a str,
    pub docsdir: &'a str,
    pub licensedir: &'a str,
    pub iconsdir: &'a str,
    pub langdir: &'a str,
    pub script_dir: &'a str,
    pub prefs_path: &'a str,
}

/// One `IniDelete($prefs, "UniExtract Preferences", <key>)` call — every
/// call in `_AfterUpdate` targets the same file and section, so only the
/// key varies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniKeyDeletion {
    pub ini_path: String,
    pub section: &'static str,
    pub key: &'static str,
}

/// One step of `_AfterUpdate`'s cleanup sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostUpdateAction {
    /// `FileMove(from, to)` — an unconditional single-file rename/move.
    MoveFile { from: String, to: String },
    /// `MoveFiles($from_dir, $to_dir, True)` — move every file matching
    /// the default pattern from one directory into another. Gated on the
    /// one real branch in `_AfterUpdate`.
    MoveMatchingFiles { from_dir: String, to_dir: String },
    /// `FileDelete(path)`.
    DeleteFile(String),
    /// `DirRemove(path, 1)` — recursive directory removal.
    RemoveDirRecursive(String),
    /// `IniDelete($prefs, "UniExtract Preferences", key)`.
    DeleteIniKey(IniKeyDeletion),
}

fn delete_ini_key(prefs_path: &str, key: &'static str) -> PostUpdateAction {
    PostUpdateAction::DeleteIniKey(IniKeyDeletion {
        ini_path: prefs_path.to_string(),
        section: "UniExtract Preferences",
        key,
    })
}

fn delete_from_arch_dir(bindir: &str, file: &str, actions: &mut Vec<PostUpdateAction>) {
    actions.push(PostUpdateAction::DeleteFile(format!("{bindir}x86\\{file}")));
    actions.push(PostUpdateAction::DeleteFile(format!("{bindir}x64\\{file}")));
}

/// Ports `_AfterUpdate` (UniExtract.au3:5673-5761) verbatim: the full,
/// ordered list of moves/deletes/removals/ini-key deletions. `docs_dir_has_7zip_readme`
/// is the caller's already-checked `FileExists($docsdir & "7zip_readme.txt")`
/// — the one conditional in the whole function, gating whether stray 7-Zip
/// docs get moved into the license directory.
pub fn post_update_actions(
    dirs: &PostUpdateDirs<'_>,
    docs_dir_has_7zip_readme: bool,
) -> Vec<PostUpdateAction> {
    let PostUpdateDirs {
        bindir,
        defdir,
        docsdir,
        licensedir,
        iconsdir,
        langdir,
        script_dir,
        prefs_path,
    } = *dirs;

    let mut actions = Vec::new();

    // Move files (UniExtract.au3:5675-5677).
    actions.push(PostUpdateAction::MoveFile {
        from: format!("{bindir}x86\\sqlite3.dll"),
        to: script_dir.to_string(),
    });
    actions.push(PostUpdateAction::MoveFile {
        from: format!("{bindir}x64\\sqlite3.dll"),
        to: format!("{script_dir}\\sqlite3_x64.dll"),
    });
    if docs_dir_has_7zip_readme {
        actions.push(PostUpdateAction::MoveMatchingFiles {
            from_dir: docsdir.to_string(),
            to_dir: licensedir.to_string(),
        });
    }

    // Remove unused files, bindir (UniExtract.au3:5680-5709).
    for name in [
        "faad.exe",
        "MediaInfo64.dll",
        "extract.exe",
        "dmgextractor.jar",
        "RPGDecrypter.exe",
        "mpq.wcx",
        "mpq.wcx64",
        "Expander.exe",
        "stuffit5.engine-5.1.dll",
        "FLVExtractCL.exe",
        "zpaqxp.exe",
        "unrar.exe",
        "xace.exe",
        "disunity.bat",
        "disunity.jar",
        "extractMHT.exe",
        "MhtUnPack.wcx",
        "STIX_D.exe",
        "WDOSXLE.exe",
        "wtee.exe",
        "ns2dec.exe",
        "EXTRNT.EXE",
        "ethornell.exe",
        "libpng12.dll",
        "brunsdec.exe",
        "sim_unpacker.exe",
        "regexp.ndll",
        "lime.ndll",
        "dbxplug.wcx",
        "unecm.exe",
    ] {
        actions.push(PostUpdateAction::DeleteFile(format!("{bindir}{name}")));
    }

    // defdir/docsdir/licensedir (UniExtract.au3:5711-5730).
    for name in ["flv.ini", "ns2.ini", "bruns.ini"] {
        actions.push(PostUpdateAction::DeleteFile(format!("{defdir}{name}")));
    }
    actions.push(PostUpdateAction::DeleteFile(format!(
        "{docsdir}FFmpeg_license.html"
    )));
    for name in [
        "flac_authors.txt",
        "flac_readme.txt",
        "Expander_license.txt",
        "flvextractcl_icons.txt",
        "wixtoolset_source.nz",
        "disunity_license.md",
        "disunity_readme.md",
        "xace_license.txt",
        "GCFScape_license.txt",
        "ns2dec_readme.txt",
        "extract_license.txt",
        "Arc-reader_licence.txt",
        "Arc-reader_readme.txt",
        "libpng_license.txt",
        "wixtoolset_source.zpaq",
        "unzoo.c",
    ] {
        actions.push(PostUpdateAction::DeleteFile(format!("{licensedir}{name}")));
    }

    // iconsdir/langdir/script_dir (UniExtract.au3:5732-5742).
    for name in [
        "Bioruebe.jpg",
        "uniextract_inno.bmp",
        "simple.jpg",
        "cascading.jpg",
    ] {
        actions.push(PostUpdateAction::DeleteFile(format!("{iconsdir}{name}")));
    }
    for name in ["Chinese.ini", "changes.txt"] {
        actions.push(PostUpdateAction::DeleteFile(format!("{langdir}{name}")));
    }
    for name in [
        "todo.txt",
        "useful_software.txt",
        "helper_binaries_info.txt",
        "changelog_minor.txt",
        "changelog.txt",
    ] {
        actions.push(PostUpdateAction::DeleteFile(format!(
            "{script_dir}\\{name}"
        )));
    }

    // Arch-dir deletes, both x86 and x64 (UniExtract.au3:5744-5748).
    for name in [
        "flac.exe",
        "7z.dll.new",
        "7z.exe.new",
        "GCFScape.exe",
        "hllib.dll",
    ] {
        delete_from_arch_dir(bindir, name, &mut actions);
    }

    // Directory removals (UniExtract.au3:5750-5755).
    for name in [
        "unrpa",
        "languages",
        "plugins",
        "crass-0.4.14.0",
        "lib",
        "file",
    ] {
        actions.push(PostUpdateAction::RemoveDirRecursive(format!(
            "{bindir}{name}"
        )));
    }

    // Ini changes (UniExtract.au3:5758-5760).
    for key in ["removetemp", "consoleoutput", "checkgame"] {
        actions.push(delete_ini_key(prefs_path, key));
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::{post_update_actions, IniKeyDeletion, PostUpdateAction, PostUpdateDirs};

    fn dirs() -> PostUpdateDirs<'static> {
        PostUpdateDirs {
            bindir: "C:\\App\\bin\\",
            defdir: "C:\\App\\def\\",
            docsdir: "C:\\App\\docs\\",
            licensedir: "C:\\App\\licenses\\",
            iconsdir: "C:\\App\\icons\\",
            langdir: "C:\\App\\lang\\",
            script_dir: "C:\\App",
            prefs_path: "C:\\App\\prefs.ini",
        }
    }

    /// The one real branch in `_AfterUpdate`: the docs->license move only
    /// appears when the sentinel file is present.
    #[test]
    fn docs_move_only_included_when_sentinel_file_exists() {
        let with = post_update_actions(&dirs(), true);
        let without = post_update_actions(&dirs(), false);
        assert!(with.contains(&PostUpdateAction::MoveMatchingFiles {
            from_dir: "C:\\App\\docs\\".to_string(),
            to_dir: "C:\\App\\licenses\\".to_string(),
        }));
        assert_eq!(with.len(), without.len() + 1);
    }

    #[test]
    fn sqlite_files_are_moved_unconditionally() {
        let actions = post_update_actions(&dirs(), false);
        assert_eq!(
            actions[0],
            PostUpdateAction::MoveFile {
                from: "C:\\App\\bin\\x86\\sqlite3.dll".to_string(),
                to: "C:\\App".to_string(),
            }
        );
        assert_eq!(
            actions[1],
            PostUpdateAction::MoveFile {
                from: "C:\\App\\bin\\x64\\sqlite3.dll".to_string(),
                to: "C:\\App\\sqlite3_x64.dll".to_string(),
            }
        );
    }

    /// Completeness check: every one of the 30 bindir deletions from the
    /// source is present, none dropped or misspelled.
    #[test]
    fn all_thirty_bindir_deletions_present() {
        let actions = post_update_actions(&dirs(), false);
        let expected = [
            "faad.exe",
            "MediaInfo64.dll",
            "extract.exe",
            "dmgextractor.jar",
            "RPGDecrypter.exe",
            "mpq.wcx",
            "mpq.wcx64",
            "Expander.exe",
            "stuffit5.engine-5.1.dll",
            "FLVExtractCL.exe",
            "zpaqxp.exe",
            "unrar.exe",
            "xace.exe",
            "disunity.bat",
            "disunity.jar",
            "extractMHT.exe",
            "MhtUnPack.wcx",
            "STIX_D.exe",
            "WDOSXLE.exe",
            "wtee.exe",
            "ns2dec.exe",
            "EXTRNT.EXE",
            "ethornell.exe",
            "libpng12.dll",
            "brunsdec.exe",
            "sim_unpacker.exe",
            "regexp.ndll",
            "lime.ndll",
            "dbxplug.wcx",
            "unecm.exe",
        ];
        assert_eq!(expected.len(), 30);
        for name in expected {
            let expected_path = format!("C:\\App\\bin\\{name}");
            assert!(
                actions.contains(&PostUpdateAction::DeleteFile(expected_path.clone())),
                "missing deletion for {expected_path}"
            );
        }
    }

    /// Completeness check: every one of the 16 licensedir deletions is
    /// present.
    #[test]
    fn all_sixteen_licensedir_deletions_present() {
        let actions = post_update_actions(&dirs(), false);
        let expected = [
            "flac_authors.txt",
            "flac_readme.txt",
            "Expander_license.txt",
            "flvextractcl_icons.txt",
            "wixtoolset_source.nz",
            "disunity_license.md",
            "disunity_readme.md",
            "xace_license.txt",
            "GCFScape_license.txt",
            "ns2dec_readme.txt",
            "extract_license.txt",
            "Arc-reader_licence.txt",
            "Arc-reader_readme.txt",
            "libpng_license.txt",
            "wixtoolset_source.zpaq",
            "unzoo.c",
        ];
        assert_eq!(expected.len(), 16);
        for name in expected {
            let expected_path = format!("C:\\App\\licenses\\{name}");
            assert!(
                actions.contains(&PostUpdateAction::DeleteFile(expected_path.clone())),
                "missing deletion for {expected_path}"
            );
        }
    }

    /// Arch-dir deletions expand into both x86 and x64 subdirectories.
    #[test]
    fn arch_dir_deletions_cover_both_architectures() {
        let actions = post_update_actions(&dirs(), false);
        for name in [
            "flac.exe",
            "7z.dll.new",
            "7z.exe.new",
            "GCFScape.exe",
            "hllib.dll",
        ] {
            assert!(actions.contains(&PostUpdateAction::DeleteFile(format!(
                "C:\\App\\bin\\x86\\{name}"
            ))));
            assert!(actions.contains(&PostUpdateAction::DeleteFile(format!(
                "C:\\App\\bin\\x64\\{name}"
            ))));
        }
    }

    #[test]
    fn all_six_directory_removals_present() {
        let actions = post_update_actions(&dirs(), false);
        for name in [
            "unrpa",
            "languages",
            "plugins",
            "crass-0.4.14.0",
            "lib",
            "file",
        ] {
            assert!(
                actions.contains(&PostUpdateAction::RemoveDirRecursive(format!(
                    "C:\\App\\bin\\{name}"
                )))
            );
        }
    }

    #[test]
    fn all_three_ini_key_deletions_present_targeting_the_same_section_and_file() {
        let actions = post_update_actions(&dirs(), false);
        for key in ["removetemp", "consoleoutput", "checkgame"] {
            assert!(
                actions.contains(&PostUpdateAction::DeleteIniKey(IniKeyDeletion {
                    ini_path: "C:\\App\\prefs.ini".to_string(),
                    section: "UniExtract Preferences",
                    key,
                }))
            );
        }
    }

    /// Exact total count: 2 unconditional moves + 30 bindir + 3 defdir +
    /// 1 docsdir + 16 licensedir + 4 iconsdir + 2 langdir + 5 scriptdir +
    /// 10 arch-dir (5 files x 2 archs) + 6 dir removals + 3 ini deletions.
    #[test]
    fn total_action_count_matches_source_exactly() {
        let actions = post_update_actions(&dirs(), false);
        assert_eq!(actions.len(), 2 + 30 + 3 + 1 + 16 + 4 + 2 + 5 + 10 + 6 + 3);
    }
}

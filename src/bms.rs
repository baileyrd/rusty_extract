//! Game-archive BMS-script lookup — `CheckGame`'s `BMS.db`-backed
//! extension-to-game disambiguation, and `BmsExtract`'s own
//! script-fetch-and-test step, capability C055.
//!
//! ```autoit
//! ; Check if game specific bms script is available
//! Local $hDB = OpenDB("BMS.db")
//! Local $aReturn[0], $iRows, $iColumns
//!
//! _SQLite_GetTable($hDB, "SELECT n.Name FROM Names n, Scripts s, Extensions e WHERE s.SID = e.EID AND s.SID = n.NID AND e.Extension= '" _
//!                       & $fileext & "' ORDER BY n.Name", $aReturn, $iRows, $iColumns)
//! _ArrayDelete($aReturn, 1)
//!
//! If $aReturn[0] > 1 Then
//!     _ArrayDelete($aReturn, 0)
//!     _ArraySort($aReturn)
//!     Local $iChoice = GUI_MethodSelectList($aReturn, t('METHOD_GAME_NOGAME'))
//!     If $iChoice > -1 Then BmsExtract($iChoice, $hDB)
//! EndIf
//!
//! _SQLite_Close()
//! _SQLite_Shutdown()
//!
//! Func GUI_MethodSelectList($aEntries, $sStandard = "", $sText = "METHOD_GAME_LABEL")
//!     If $sMethodSelectOverride > 0 Then
//!         Local $iLen = UBound($aEntries)
//!         Local $iIndex = $sMethodSelectOverride - 1
//!         If $iIndex < 1 Then Return 0
//!
//!         If $iLen < $iIndex Then
//!             Cout("Invalid method select override: index is " & $iIndex & ", but only " & $iLen & " choices available")
//!         Else
//!             Return $aEntries[$iIndex - 1]
//!         EndIf
//!     EndIf
//!
//!     Local $sSelection = 0
//!     If $silentmode Then Return $sSelection
//!     ; ... GUI list dialog (out of scope, D001) ...
//! EndFunc
//!
//! Func BmsExtract($sName, $hDB = 0)
//!     If Not $sName Then Return
//!     If $hDB == 0 Then $hDB = OpenDB("BMS.db")
//!
//!     If $hDB Then
//!         Local $aReturn[0], $iRows, $iColumns
//!         _SQLite_GetTable($hDB, Cout("SELECT s.Script FROM Scripts s, Names n WHERE s.SID = n.NID AND Name = '" & $sName & "'"), $aReturn, $iRows, $iColumns)
//!
//!         ; Write script to file and execute it
//!         Local $hFile = FileOpen($bms, $FO_OVERWRITE)
//!         FileWrite($hFile, $aReturn[2])
//!         FileClose($hFile)
//!         Local $return = FetchStdout($quickbms & ' -l "' & $bms & '" "' & $file & '"', $filedir, @SW_HIDE, -1)
//!
//!         If Not StringInStr($return, "0 files found") And Not StringInStr($return, "Error") And Not StringInStr($return, "invalid") _
//!         And Not StringInStr($return, "expected: ") And $return <> "" Then
//!             extract($TYPE_QBMS, $sName & " " & t('TERM_PACKAGE'))
//!         EndIf
//!     EndIf
//!
//!     terminate($STATUS_FAILED, $filenamefull, $sName, $sName)
//! EndFunc
//! ```
//!
//! **The once-blocking `_SQLite_GetTable` array-shape question is now
//! resolved against AutoIt's own documentation**, not guessed: `$aResult`
//! is a flat array where `$aResult[0]` holds `($iRows + 1) * $iColumns`
//! (not the row count itself), followed by `$iColumns` header entries,
//! then the data in row-major order. Both queries here select exactly
//! one column, so `$iColumns` is always `1` and `$aResult[0]` is always
//! `$iRows + 1`.
//!
//! **Preserved quirk — the `> 1` gate, explained.** After
//! `_ArrayDelete($aReturn, 1)` removes the header, `$aReturn[0]` still
//! holds the pre-deletion total (`$iRows + 1` — deleting index 1 doesn't
//! touch index 0). So `$aReturn[0] > 1` is really testing `$iRows + 1 >
//! 1`, i.e. `$iRows > 0` — "at least one candidate exists", not "more
//! than one". [`sql_lookup_outcome`] takes the already-separated row
//! count directly and applies the equivalent `rows > 0` check, rather
//! than re-deriving it from a re-inflated total.
//!
//! **Scope — SQLite access, `quickbms`/GUI plugin invocation, and
//! process termination are real I/O, out of scope everywhere in this
//! crate.** [`sql_lookup_outcome`] and [`decide_game_choice`] take
//! already-fetched/separated data (`OpenDB`/`_SQLite_GetTable`/
//! `_SQLite_Close`/`_SQLite_Shutdown` themselves are real database I/O);
//! [`should_attempt_bms_extraction`] takes the already-captured
//! `quickbms -l` probe output. The actual `.bms`-script write
//! (`FileWrite($hFile, $aReturn[2])`), the `quickbms -K` extraction
//! invocation, and `BmsExtract`'s own recursive `extract($TYPE_QBMS,
//! ...)` call reuse `extract::qbms::resolve_plugin_path`'s existing
//! `$bms`-fallback case, not duplicated here. `terminate($STATUS_FAILED,
//! ...)` is `crate::status::Status::Failed`, the same status vocabulary
//! used throughout this crate.
//!
//! **A pre-existing SQL-injection property of the source, preserved
//! exactly, not "fixed".** Both queries splice `$fileext`/`$sName`
//! directly into the SQL text with no escaping or parameterization
//! (UniExtract.au3:2025-2026,3552) — [`check_game_query`]/
//! [`bms_script_query`] reproduce the exact same string concatenation,
//! since a caller needs the literal query text `_SQLite_GetTable`
//! receives, not a hardened rewrite that would no longer match.

/// What `CheckGame`'s SQLite lookup resolves to (UniExtract.au3:2019,
/// 2028-2033): either no game entry matches this file extension, or at
/// least one does. `rows` is `$iRows`, and `game_names` is the `rows`
/// actual `Names.Name` values already separated from the header/count —
/// see the module doc comment for why `$aReturn[0] > 1` and `rows > 0`
/// are the same check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlLookupOutcome {
    /// No candidate game archives match this file extension.
    NoMatch,
    /// At least one candidate exists; the names are sorted
    /// (`_ArraySort`), ready for [`decide_game_choice`].
    Candidates(Vec<String>),
}

/// Ports the row-count gate plus `_ArraySort($aReturn)`
/// (UniExtract.au3:2028-2032).
pub fn sql_lookup_outcome(rows: u32, game_names: &[String]) -> SqlLookupOutcome {
    if rows == 0 {
        return SqlLookupOutcome::NoMatch;
    }
    let mut names = game_names.to_vec();
    names.sort();
    SqlLookupOutcome::Candidates(names)
}

/// What `GUI_MethodSelectList` resolves to (UniExtract.au3:7562-7578).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameChoiceOutcome {
    /// No BMS script gets auto-selected this run — reached by an
    /// override of exactly `1` (explicitly picking the "not a game
    /// archive" list entry, position 1 in the combined
    /// standard-plus-candidates list the real GUI shows); by no
    /// override with the run silent; or by an override naming an index
    /// beyond `candidates`' length while the run is silent (an
    /// out-of-range override just logs an error and falls through to
    /// the same path plain "no override" takes, rather than failing
    /// outright).
    NoSelection,
    /// A valid override (`>= 2`) picked one specific candidate outright.
    Selected(String),
    /// No (valid) override, and the run isn't silent: the list-selection
    /// GUI dialog itself, out of scope (deferred GUI subsystem, D001).
    PromptInteractive,
}

/// Ports `GUI_MethodSelectList`'s pre-dialog branch selection.
/// `method_select_override` is `None` for `$sMethodSelectOverride`'s
/// default `0` (no override), matching `method_select::
/// decide_method_selection`'s own representation — reused here for
/// consistency, not duplicated as a new shape. Note the override's
/// 1-indexing is shifted by one relative to `GUI_MethodSelect`'s (C053):
/// override `1` means "not a game" (position 1 in the real dialog's
/// combined list), override `2` means `candidates[0]`, and so on.
pub fn decide_game_choice(
    method_select_override: Option<&str>,
    candidates: &[String],
    silent_mode: bool,
) -> GameChoiceOutcome {
    if let Some(digits) = method_select_override {
        if let Ok(override_num) = digits.parse::<u32>() {
            if override_num > 0 {
                let index = override_num - 1;
                if index < 1 {
                    return GameChoiceOutcome::NoSelection;
                }
                if candidates.len() as u32 >= index {
                    let zero_based = (index - 1) as usize;
                    return GameChoiceOutcome::Selected(candidates[zero_based].clone());
                }
                // Out-of-range override: logged in the source (not
                // modeled, real I/O), falls through below.
            }
        }
    }
    if silent_mode {
        GameChoiceOutcome::NoSelection
    } else {
        GameChoiceOutcome::PromptInteractive
    }
}

/// Ports `BmsExtract`'s success-indicator classification
/// (UniExtract.au3:3559-3561): the `quickbms -l` probe's captured output
/// must contain none of four failure markers and be non-empty. Every
/// `StringInStr` here is bare (case-insensitive, AutoIt's documented
/// default), matching the case-insensitive handling used throughout
/// `extract::qbms`'s own probe classifiers.
pub fn should_attempt_bms_extraction(quickbms_list_output: &str) -> bool {
    let lower = quickbms_list_output.to_lowercase();
    !lower.contains("0 files found")
        && !lower.contains("error")
        && !lower.contains("invalid")
        && !lower.contains("expected: ")
        && !quickbms_list_output.is_empty()
}

/// Builds `CheckGame`'s lookup query exactly (UniExtract.au3:2025-2026).
/// See the module doc comment for why `fileext` is spliced in
/// unescaped, matching the source.
pub fn check_game_query(fileext: &str) -> String {
    format!(
        "SELECT n.Name FROM Names n, Scripts s, Extensions e WHERE s.SID = e.EID AND s.SID = n.NID AND e.Extension= '{fileext}' ORDER BY n.Name"
    )
}

/// Builds `BmsExtract`'s script-fetch query exactly
/// (UniExtract.au3:3552). See the module doc comment for why `name` is
/// spliced in unescaped, matching the source.
pub fn bms_script_query(name: &str) -> String {
    format!("SELECT s.Script FROM Scripts s, Names n WHERE s.SID = n.NID AND Name = '{name}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_rows_means_no_match() {
        assert_eq!(sql_lookup_outcome(0, &[]), SqlLookupOutcome::NoMatch);
    }

    #[test]
    fn one_or_more_rows_sorts_and_returns_candidates() {
        let names = vec!["Zelda".to_string(), "Ace Combat".to_string()];
        assert_eq!(
            sql_lookup_outcome(2, &names),
            SqlLookupOutcome::Candidates(vec!["Ace Combat".to_string(), "Zelda".to_string()])
        );
    }

    /// Parity test for capability C055: exactly one candidate still
    /// counts as "at least one" (`rows > 0`, derived from the source's
    /// own `> 1` gate on the pre-header-delete total — see module doc
    /// comment).
    #[test]
    fn single_candidate_still_counts_as_a_match() {
        let names = vec!["Ace Combat".to_string()];
        assert_eq!(
            sql_lookup_outcome(1, &names),
            SqlLookupOutcome::Candidates(vec!["Ace Combat".to_string()])
        );
    }

    /// Internal consistency check against AutoIt's own documented
    /// `_SQLite_GetTable` array shape (`$aResult[0] == (rows+1) *
    /// columns`, `columns == 1` for this query): the surviving gate
    /// value after deleting index 1 is `rows + 1`, and `rows + 1 > 1`
    /// is exactly `rows > 0` for every non-negative `rows`.
    #[test]
    fn row_count_gate_matches_raw_array_shape_arithmetic() {
        for rows in 0..5u32 {
            let raw_gate_value = rows + 1; // $aReturn[0] after the header delete
            let source_check = raw_gate_value > 1;
            let ported_check = rows > 0;
            assert_eq!(source_check, ported_check, "mismatch at rows={rows}");
        }
    }

    #[test]
    fn override_of_one_means_not_a_game() {
        let candidates = vec!["Ace Combat".to_string(), "Zelda".to_string()];
        assert_eq!(
            decide_game_choice(Some("1"), &candidates, false),
            GameChoiceOutcome::NoSelection
        );
    }

    #[test]
    fn override_of_two_selects_first_candidate() {
        let candidates = vec!["Ace Combat".to_string(), "Zelda".to_string()];
        assert_eq!(
            decide_game_choice(Some("2"), &candidates, false),
            GameChoiceOutcome::Selected("Ace Combat".to_string())
        );
    }

    #[test]
    fn override_of_three_selects_second_candidate() {
        let candidates = vec!["Ace Combat".to_string(), "Zelda".to_string()];
        assert_eq!(
            decide_game_choice(Some("3"), &candidates, false),
            GameChoiceOutcome::Selected("Zelda".to_string())
        );
    }

    /// Parity test for capability C055: an override past the candidate
    /// list's length falls through to the same path as no override at
    /// all, rather than erroring out.
    #[test]
    fn out_of_range_override_falls_through_to_silent_mode() {
        let candidates = vec!["Ace Combat".to_string()];
        assert_eq!(
            decide_game_choice(Some("5"), &candidates, true),
            GameChoiceOutcome::NoSelection
        );
    }

    #[test]
    fn out_of_range_override_falls_through_to_interactive_prompt() {
        let candidates = vec!["Ace Combat".to_string()];
        assert_eq!(
            decide_game_choice(Some("5"), &candidates, false),
            GameChoiceOutcome::PromptInteractive
        );
    }

    #[test]
    fn no_override_silent_mode_means_no_selection() {
        let candidates = vec!["Ace Combat".to_string()];
        assert_eq!(
            decide_game_choice(None, &candidates, true),
            GameChoiceOutcome::NoSelection
        );
    }

    #[test]
    fn no_override_interactive_mode_prompts() {
        let candidates = vec!["Ace Combat".to_string()];
        assert_eq!(
            decide_game_choice(None, &candidates, false),
            GameChoiceOutcome::PromptInteractive
        );
    }

    #[test]
    fn should_attempt_extraction_when_output_is_clean() {
        assert!(should_attempt_bms_extraction(
            "Offset  Filename\n0  file.dat"
        ));
    }

    #[test]
    fn should_not_attempt_extraction_on_any_failure_marker() {
        assert!(!should_attempt_bms_extraction("0 files found"));
        assert!(!should_attempt_bms_extraction("Error: script crashed"));
        assert!(!should_attempt_bms_extraction("invalid script"));
        assert!(!should_attempt_bms_extraction("expected: a number"));
        assert!(!should_attempt_bms_extraction(""));
    }

    /// Parity test for capability C055: every failure-marker check is
    /// case-insensitive (bare `StringInStr`).
    #[test]
    fn failure_markers_are_case_insensitive() {
        assert!(!should_attempt_bms_extraction("ERROR"));
        assert!(!should_attempt_bms_extraction("EXPECTED: something"));
    }

    #[test]
    fn check_game_query_matches_source() {
        assert_eq!(
            check_game_query("zip"),
            "SELECT n.Name FROM Names n, Scripts s, Extensions e WHERE s.SID = e.EID AND s.SID = n.NID AND e.Extension= 'zip' ORDER BY n.Name"
        );
    }

    #[test]
    fn bms_script_query_matches_source() {
        assert_eq!(
            bms_script_query("Ace Combat"),
            "SELECT s.Script FROM Scripts s, Names n WHERE s.SID = n.NID AND Name = 'Ace Combat'"
        );
    }
}

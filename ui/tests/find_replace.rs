use scriptcast_ui::{
    find::{self, Matcher, Mode, Options},
    script::ScriptTurn,
};

fn turn(index: usize, body: &str) -> ScriptTurn {
    ScriptTurn {
        index,
        turn_id: format!("turn-{index}"),
        speaker_id: "alice".into(),
        display_name: "Alice".into(),
        header_line: index * 3 + 1,
        body_start: index * 3 + 2,
        body_end: index * 3 + 2,
        body: body.into(),
    }
}

#[test]
fn case_whole_word_and_unicode_byte_offsets() {
    let mut options = Options::default();
    let text = "Café café cafés CAFÉ";
    let ranges = Matcher::new("café", &options).unwrap().ranges(text);
    assert_eq!(ranges.len(), 4);
    assert_eq!(&text[ranges[3].clone()], "CAFÉ");
    options.match_case = true;
    assert_eq!(
        Matcher::new("café", &options).unwrap().ranges(text).len(),
        2
    );
    options.whole_word = true;
    assert_eq!(
        Matcher::new("café", &options).unwrap().ranges(text).len(),
        1
    );
    options.match_case = false;
    assert_eq!(
        Matcher::new("café", &options).unwrap().ranges(text).len(),
        3
    );
}

#[test]
fn regex_invalid_empty_and_literal_queries() {
    let mut options = Options::default();
    assert_eq!(Matcher::new("[", &options).unwrap().ranges("a[b").len(), 1);
    options.regular_expression = true;
    assert!(Matcher::new("[", &options).is_err());
    assert_eq!(
        Matcher::new(r"c.t\b", &options)
            .unwrap()
            .ranges("cat cot cats")
            .len(),
        2
    );
    assert!(
        Matcher::new("", &options)
            .unwrap()
            .ranges("anything")
            .is_empty()
    );
}

#[test]
fn displayed_clip_tags_are_excluded_and_offsets_map_back_to_body() {
    let mut options = Options::default();
    let body = "  [clip:racism] Racism and racism.  ";
    let matcher = Matcher::new("racism", &options).unwrap();
    let found = find::occurrences(3, body, &matcher, &options);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].displayed, 0..6);
    assert_eq!(&body[found[0].body.clone().unwrap()], "Racism");
    options.include_clips = false;
    assert!(find::occurrences(3, body, &matcher, &options).is_empty());
    options.include_clips = true;
    let matcher = Matcher::new("one two", &options).unwrap();
    let found = find::occurrences(0, "one [clip:x]two", &matcher, &options);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].body, None);
}

#[test]
fn navigation_wraps_both_ways_and_handles_empty_or_stale_position() {
    assert_eq!(find::navigate(None, 3, false), Some(0));
    assert_eq!(find::navigate(None, 3, true), Some(2));
    assert_eq!(find::navigate(Some(2), 3, false), Some(0));
    assert_eq!(find::navigate(Some(0), 3, true), Some(2));
    assert_eq!(find::navigate(Some(0), 1, false), Some(0));
    assert_eq!(find::navigate(Some(4), 0, false), None);
    assert_eq!(find::navigate(Some(9), 3, false), Some(0));
}

#[test]
fn turns_keep_speaker_number_and_text_matching() {
    let options = Options {
        mode: Mode::Turns,
        ..Default::default()
    };
    for query in ["WORLD", "alice", "007", ""] {
        assert!(
            Matcher::new(query, &options)
                .unwrap()
                .matches_turn("Hello world", "Alice", 7)
        );
    }
    assert!(
        !Matcher::new("nope", &options)
            .unwrap()
            .matches_turn("Hello world", "Alice", 7)
    );
}

#[test]
fn planner_counts_skips_and_builds_each_body_without_touching_tags() {
    let options = Options::default();
    let matcher = Matcher::new("cat", &options).unwrap();
    let turns = vec![
        turn(0, "Cat cat."),
        turn(1, "[clip:cat] cat."),
        turn(2, "dog."),
    ];
    let plan = find::plan(&turns, &matcher, &options, "dog", None);
    assert_eq!(
        (plan.occurrences, plan.edits.len(), plan.skipped),
        (3, 2, 1)
    );
    assert_eq!(plan.edits[0].1, "dog dog.");
    assert_eq!(plan.edits[1].1, "[clip:cat] dog.");
    assert!(plan.preview().contains("1 skipped"));
    let current = find::occurrences(0, &turns[0].body, &matcher, &options).remove(1);
    let single = find::plan(&turns, &matcher, &options, "dog", Some(&current));
    assert_eq!((single.occurrences, single.edits.len()), (1, 1));
    assert_eq!(single.edits[0].1, "Cat dog.");
    let excluded = find::plan(
        &turns,
        &matcher,
        &Options {
            include_clips: false,
            ..options
        },
        "dog",
        None,
    );
    assert_eq!((excluded.occurrences, excluded.skipped), (2, 0));
}

#[test]
fn regex_capture_replacement_and_spanning_tag_protection() {
    let options = Options {
        regular_expression: true,
        ..Default::default()
    };
    let turns = vec![turn(0, "cat 12 cat 34"), turn(1, "[clip:cat] cat")];
    let matcher = Matcher::new(r"cat (\d+)", &options).unwrap();
    let plan = find::plan(&turns, &matcher, &options, "${1} dogs", None);
    assert_eq!(plan.edits[0].1, "12 dogs 34 dogs");
    let matcher = Matcher::new(r"cat\] cat", &options).unwrap();
    let plan = find::plan(&turns, &matcher, &options, "gone", None);
    assert_eq!(plan.skipped, 1);
    assert!(plan.edits.is_empty());
}

#[test]
fn options_round_trip_and_old_preferences_defaults() {
    let defaults: Options = serde_json::from_str("{}").unwrap();
    assert_eq!(defaults, Options::default());
    let options = Options {
        mode: Mode::Turns,
        match_case: true,
        whole_word: true,
        regular_expression: true,
        include_clips: false,
    };
    let saved = serde_json::to_string(&options).unwrap();
    assert_eq!(serde_json::from_str::<Options>(&saved).unwrap(), options);
}

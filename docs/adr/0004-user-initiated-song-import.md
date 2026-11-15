# ADR 0004: Song content enters only by user action and stays on the device

- Status: Proposed
- Date: 2026-11-15

## Context
Users want to search for songs online and bring in their chords and lyrics. Lyrics are copyrighted, chord sheets usually include them, and the major chord sites' terms generally forbid scraping and reuse. App Review rejects apps that pull third-party copyrighted content. See [song-import.md §2](../song-import.md#2-legal-and-terms-of-service-constraints-read-first).

## Decision
1. No crawler, no background fetching, no project-run server holding song content.
2. Phase 1 imports only from paste, files and the user's own charts.
3. Web import (Share extension, then URL) and search come later, behind a legal review, and only as explicit one-page user actions with local storage.
4. A `SongSource` provider protocol keeps the door open for licensed catalogues.
5. No copyrighted lyrics in the repository, including test fixtures.

## Consequences
The v1 "search the internet" experience is weaker than a scraper would give, and is mitigated by fast paste import, a strong parser and a public-domain starter library. The product stays distributable and the licensed path stays open.

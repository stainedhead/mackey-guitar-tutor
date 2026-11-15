# 02. Tuner

Phase: P1 (v1). Design: [learning-features.md §1](../learning-features.md#1-built-in-tuner-v1), [ADR 0006](../adr/0006-on-device-audio-analysis.md), [platforms.md §3](../platforms.md#3-ipad).

## 1. Purpose

Get the guitar in tune quickly and accurately before practice, in any supported tuning, with no extra hardware.

## 2. User stories

| ID | Story |
|----|-------|
| US-TU-1 | As a beginner, I play each open string and the tuner tells me which string it is and whether to tighten or loosen. |
| US-TU-2 | As a player in DADGAD, the tuner targets DADGAD, not standard. |
| US-TU-3 | As a player without a quiet room, I can tune by ear against reference tones. |

## 3. Functional requirements

- TU-1 Chromatic mode MUST show: detected note name and octave, cents offset from the nearest equal-tempered pitch (−50 to +50), a needle or strobe-style bar, and the frequency in Hz (small text).
- TU-1.1 "In tune" is within ±3 cents, held for 300 ms; the display turns green with a check mark and a "In tune" label (never colour alone).
- TU-1.2 Detection range: 60 Hz to 1,400 Hz (covers drop tunings to the 24th fret of string 1).
- TU-1.3 Readings update at least 20 times per second; displayed value is smoothed (median of the last 3 frames) so the needle does not jitter.
- TU-1.4 When no stable pitch is found for 500 ms, show "Play a single string" and park the needle at centre in a neutral colour.
- TU-2 String mode (default): shows the six target strings for the current tuning with capo applied, auto-selects the nearest string by pitch, and shows arrows "Tighten" / "Loosen" with the cents offset. Tapping a string locks the target to it.
- TU-2.1 If the detected pitch is more than 3 semitones from every target string, show "Far from any string: check the tuning setting" and the nearest string.
- TU-2.2 Each string shows a tick once it has been in tune during this tuning session.
- TU-3 Reference pitch A4 settable from 415 to 466 Hz in 1 Hz steps, default 440, saved.
- TU-4 Reference tones: tapping the speaker icon on a string plays its target pitch for 3 seconds (looping until tapped again) using the steel-guitar sound.
- TU-5 With a capo, target pitches are the sounding pitches of the open strings at the capo.
- TU-6 Input: the default input device; a device picker lists every CoreAudio input; input level meter shown. On iPad the audio session is put in measurement mode while the tuner is open (no automatic gain).
- TU-7 Opening the tuner pauses the transport and selection playback. Closing it restores the previous audio session mode.

## 4. Edge cases and error states

| ID | Situation | Required behaviour |
|----|-----------|--------------------|
| E-TU-1 | Microphone permission not yet asked | Explain screen: "Mackey uses the microphone only to hear your guitar. Nothing is recorded or sent anywhere." with a Continue button that triggers the OS prompt. |
| E-TU-2 | Permission denied | Tuner shows "Microphone access is off" with a button that opens System Settings (Mac) or Settings (iPad), and reference tones still work. |
| E-TU-3 | Input level too low for 3 seconds | "Too quiet: move closer or turn up your input". |
| E-TU-4 | Input clipping | "Too loud: turn down your input". |
| E-TU-5 | Input device disconnected while open | Switch to the default input and show a notice. |
| E-TU-6 | Two strings ringing | Show the strongest pitch; if confidence is low, TU-1.4 message. |
| E-TU-7 | Mac rebuild (ad-hoc signature) re-asks permission | Expected (see platforms.md §2); E-TU-1 flow appears again. |

## 5. Acceptance criteria

| ID | Given | When | Then |
|----|-------|------|------|
| AC-TU-1.1 | Synthetic sine at 440.0 Hz, A4 = 440 | Tuner input | Shows A4, 0 ± 1 cent, "In tune" within 400 ms. |
| AC-TU-1.2 | Recorded guitar samples of each open string, detuned by −20, −5, +5, +20 cents | Tuner input | Reported offset within ±3 cents of truth for every sample (test fixture set). |
| AC-TU-1.3 | Synthetic 73.42 Hz (D2) | Chromatic mode | Shows D2. |
| AC-TU-2.1 | Tuning DADGAD, input 146.83 Hz (D3) | String mode | Selects string 4 (D), in tune. |
| AC-TU-3.1 | A4 = 442 | Input 440 Hz | Shows A4, about −8 cents. |
| AC-TU-5.1 | Capo 2, standard tuning | String mode | Targets F#2 B2 E3 A3 C#4 F#4. |
| AC-E-TU-2 | Permission denied | Open tuner | Message and settings button shown; no crash. *(manual on device)* |

## 6. Out of scope

Polyphonic "strum all strings" tuning, intonation adjustment helper, alternate temperaments.

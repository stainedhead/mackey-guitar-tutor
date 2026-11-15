# Troubleshooting

## macOS says the app cannot be opened

The app is signed ad hoc (no Apple Developer ID), so Gatekeeper blocks it the first time with a message that it "cannot be opened" or "is from an unidentified developer".

1. Try to open the app once so macOS records the attempt.
2. Open **System Settings > Privacy & Security** and scroll down to the Security section.
3. Next to the message about "Mackey Guitar Tutor", choose **Open Anyway** and confirm with your password or Touch ID.

You only need to do this once per build. An app you built yourself with `make app` on the same Mac normally opens without the prompt.

## The tuner says "Microphone access is off"

macOS keeps one permission per app. Open **System Settings > Privacy & Security > Microphone**, switch Mackey Guitar Tutor on, then return to the Tuner. The **Open System Settings** button on the Tuner screen jumps straight there.

**Asked again after a rebuild.** Each ad-hoc build has a new code signature, so macOS may treat it as a different app and ask again, or keep a stale "off" entry. If the switch is on but the tuner still reports no access, quit the app, remove its entry with `tccutil reset Microphone dev.stainedhead.mackey` in Terminal, and open the Tuner again to get a fresh prompt.

## "No microphone found"

The Tuner found no input device. Connect a microphone or audio interface, or pick one in the **Input** menu. The reference tones work without a microphone.

## The tuner says "This input is not delivering audio at a normal rate"

The selected input is sending audio much faster than real time (a virtual or test device that is not tied to a clock), so the tuner stopped listening instead of using the whole processor. Choose another input in the **Input** menu, or close and reopen the Tuner.

## The tuner says "Too quiet" or "No stable pitch"

"Too quiet": move the guitar closer to the microphone or raise the input level in System Settings > Sound > Input. "No stable pitch": mute the other strings and pluck one string, then let it ring.

## No sound, or "No audio output"

The app could not open an output device. Check that an output (speakers, headphones) is selected in System Settings > Sound > Output and that **Mute** is off and **Volume** is above zero in [Settings](settings.md#sound). The message appears once per session; press Play again after fixing the device. The rest of the app keeps working without sound.

## A chord says it is not understood

Chord symbols are spelled `Cmaj7`, `Am7`, `Bb`, `F#m`, `D/F#`, `Gsus4`. The message under the Chord field says what it could not read and often suggests the right spelling. Roman numerals (`vi`, `V7`) depend on the key shown.

## Settings are lost or odd

Delete `settings.json` in `~/Library/Application Support/dev.stainedhead.mackey/` to return to the defaults. A damaged file is set aside as `settings.json.bak-<time>` automatically, and you are told once. The log is in the `logs` folder beside it.

## Where to report a problem

Open an issue at <https://github.com/stainedhead/mackey-guitar-tutor/issues> with the app's log file and what you were doing.

Back to [user-docs](README.md).

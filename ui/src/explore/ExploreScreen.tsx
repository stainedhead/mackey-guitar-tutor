import { useCallback, useEffect } from 'react';
import { FretboardPanel } from '../fretboard/FretboardPanel';
import type { Swipe } from '../gestures';
import type { LayoutInfo } from '../layout/useLayout';
import { useStore } from '../state/context';
import { useUiState } from '../state/store';
import { ChordField } from './ChordField';
import { ChordStrip } from './ChordStrip';
import { Toolbar } from './Toolbar';
import { ViewControls } from './ViewControls';
import { VoicingList } from './VoicingList';

/** The Explore screen: a thin renderer of the snapshot Rust sent. Every control calls a command. */
export function ExploreScreen({ layout }: { layout: LayoutInfo }) {
  const store = useStore();
  const snapshot = useUiState(store, (s) => s.snapshot);
  const zoom = useUiState(store, (s) => s.zoom);
  const draft = useUiState(store, (s) => s.draftChord);
  const settings = useUiState(store, (s) => s.settings);

  // Each layout has its own default view of the neck (0-15 Mac, 0-12 iPad landscape, 0-7 portrait).
  useEffect(() => {
    store.setZoom(layout.defaultRange);
  }, [store, layout.defaultRange]);

  const onSwipe = useCallback(
    (swipe: Swipe) => {
      if (swipe.fingers === 2) {
        if (swipe.direction === 'left' || swipe.direction === 'up')
          void store.cycle('PositionNext');
        if (swipe.direction === 'right' || swipe.direction === 'down')
          void store.cycle('PositionPrev');
        return;
      }
      if (swipe.direction === 'left') void store.cycle('VoicingNext');
      if (swipe.direction === 'right') void store.cycle('VoicingPrev');
    },
    [store],
  );

  const pickVoicing = useCallback(
    (index: number) => {
      const selected = snapshot?.view.voicing_list.findIndex((v) => v.selected) ?? 0;
      const count = snapshot?.view.voicing_list.length ?? 1;
      const steps = (index - selected + count) % count;
      void (async () => {
        for (let i = 0; i < steps; i += 1) await store.cycle('VoicingNext');
      })();
    },
    [snapshot, store],
  );

  if (!snapshot) return <p className="loading">Loading</p>;
  const { panel, view } = snapshot;
  const hasChord = panel.selection.chord !== null;
  const text = draft ?? panel.chord_text;
  return (
    <div className="explore" data-testid="explore">
      <Toolbar
        panel={panel}
        hasChord={hasChord}
        onKey={(tonic, mode) => void store.selectKey(tonic, mode)}
        onTuning={(t) => void store.selectTuning(t)}
        onCapo={(c) => void store.setCapo(c)}
        onLabels={(l) => void store.setLabels(l)}
        onView={(v) => void store.setViewType(v)}
        onKeyStep={(d) => void store.cycle(d === 'next' ? 'KeyNext' : 'KeyPrev')}
        onRelative={() => void store.cycle('RelativeToggle')}
      />
      <div className="explore-main">
        <div className="explore-left">
          <ChordStrip
            panel={panel}
            badge={view.badge}
            onSelect={(d) => void store.selectDiatonic(d)}
            onSevenths={(on) => void store.setOption({ ShowSevenths: on })}
            onStep={(d) => void store.cycle(d === 'next' ? 'ChordNext' : 'ChordPrev')}
          />
          <ChordField
            text={text}
            status={panel.chord_status}
            onChange={(t) => {
              store.setDraftChord(t);
            }}
            onSubmit={(t) => void store.selectChord(t)}
          />
          <ViewControls
            panel={panel}
            view={view}
            onCycle={(a) => void store.cycle(a)}
            onOption={(o) => void store.setOption(o)}
            onOverlay={(o) => void store.setOverlay(o)}
          />
          <div className="play-row">
            <button type="button" className="primary" onClick={() => void store.play()}>
              Play
            </button>
            <span className="hint">Space</span>
          </div>
        </div>
        <VoicingList view={view} onPick={pickVoicing} />
      </div>
      {zoom && (
        <FretboardPanel
          view={view}
          zoom={zoom}
          minFretWidth={layout.minFretWidth}
          showFretNumbers={settings?.display.show_fret_numbers ?? true}
          touch={layout.touch}
          onZoom={(r) => {
            store.setZoom(r);
          }}
          onVisible={(r) => void store.setVisible(r)}
          onPlayNote={(s, f) => void store.playNote(s, f)}
          onCellClick={(s, f) => {
            void store.selectNote(s, f);
            void store.playNote(s, f);
          }}
          onSwipe={onSwipe}
        />
      )}
    </div>
  );
}

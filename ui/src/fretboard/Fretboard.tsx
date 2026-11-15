import { useMemo, useRef, useState, type KeyboardEvent } from 'react';
import type { FretboardView } from '../bindings/FretboardView';
import type { ViewPosition } from '../bindings/ViewPosition';
import { formatNote, labelFor } from './format';
import {
  LABEL_MARGIN,
  NUMBERS_HEIGHT,
  STRING_GAP,
  VERTICAL_PADDING,
  layoutNeck,
  type NeckLayout,
} from './geometry';
import { NoteMarker } from './NoteMarker';
import { Tooltip } from './Tooltip';

interface Props {
  view: FretboardView;
  scaleLength: number;
  minFretWidth: number;
  showFretNumbers: boolean;
  onPlayNote: (string: number, fret: number) => void;
  onCellClick: (string: number, fret: number) => void;
}

/** Marker radius and label size: 11 pt text (14.5 px) inside a marker that fits the fret. */
function markerSizes(minFretWidth: number): { radius: number; fontSize: number } {
  return { radius: Math.min(13, minFretWidth / 2 - 0.5), fontSize: 14.5 };
}

/** Left-to-right, then top-to-bottom order of the markers, for arrow-key navigation. */
function neighbour(
  order: readonly { key: string; x: number; y: number }[],
  from: string,
  key: string,
): string | undefined {
  const index = order.findIndex((o) => o.key === from);
  const current = order[index];
  if (!current) return undefined;
  if (key === 'ArrowRight' || key === 'ArrowLeft') {
    const step = key === 'ArrowRight' ? 1 : -1;
    const sorted = [...order].sort((a, b) => a.x - b.x || a.y - b.y);
    const at = sorted.findIndex((o) => o.key === from);
    return sorted[at + step]?.key;
  }
  const column = order.filter((o) => o.x === current.x).sort((a, b) => a.y - b.y);
  const at = column.findIndex((o) => o.key === from);
  return column[at + (key === 'ArrowDown' ? 1 : -1)]?.key;
}

const keyOf = (p: ViewPosition): string => `${p.string.toString()}-${p.fret.toString()}`;

/**
 * The fretboard, drawn only from a `FretboardView`: strings, frets with 12th-root-of-2 spacing,
 * nut, inlays, fret numbers, string labels, role shapes, open and muted markers, barres, capo
 * dimming, and the left-handed mirror and string flip. Every position is a focusable element
 * with the label Rust made.
 */
export function Fretboard({
  view,
  scaleLength,
  minFretWidth,
  showFretNumbers,
  onPlayNote,
  onCellClick,
}: Props) {
  const layout: NeckLayout = useMemo(
    () =>
      layoutNeck({
        fretCount: view.fret_count,
        capo: view.capo,
        scaleLength,
        minFretWidth,
        mirrored: view.mirrored,
        flipped: view.flipped,
      }),
    [view.fret_count, view.capo, scaleLength, minFretWidth, view.mirrored, view.flipped],
  );
  const { radius, fontSize } = markerSizes(minFretWidth);
  const [hover, setHover] = useState<string | null>(null);
  const refs = useRef(new Map<string, SVGGElement>());
  const placed = view.positions.map((p) => ({
    p,
    key: keyOf(p),
    x: layout.xOfFret(p.fret, p.is_open),
    y: layout.yOfString(p.string),
  }));
  const hovered = placed.find((m) => m.key === hover);
  const middleY = VERTICAL_PADDING + STRING_GAP * 2.5;
  const neckBottom = VERTICAL_PADDING * 2 + STRING_GAP * 5;

  const onKeyDown = (event: KeyboardEvent<SVGGElement>, key: string): void => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      event.stopPropagation();
      const found = placed.find((m) => m.key === key);
      if (found) onPlayNote(found.p.string, found.p.fret);
      return;
    }
    if (!event.key.startsWith('Arrow')) return;
    const next = neighbour(placed, key, event.key);
    event.preventDefault();
    event.stopPropagation();
    if (next) refs.current.get(next)?.focus();
  };

  return (
    <svg
      className="neck"
      data-testid="neck"
      width={layout.width}
      height={layout.height}
      viewBox={`0 0 ${layout.width.toString()} ${layout.height.toString()}`}
      role="group"
      aria-label={`Fretboard. ${view.headline}`}
      data-mirrored={view.mirrored}
      data-flipped={view.flipped}
      data-capo={view.capo}
    >
      <rect
        className="neck-wood"
        x={Math.min(layout.wireX(0), layout.wireX(view.fret_count))}
        y={VERTICAL_PADDING - 14}
        width={Math.abs(layout.wireX(view.fret_count) - layout.wireX(0))}
        height={STRING_GAP * 5 + 28}
        onClick={(e) => {
          const box = e.currentTarget.ownerSVGElement?.getBoundingClientRect();
          if (!box) return;
          const cell = layout.cellAt(e.clientX - box.left, e.clientY - box.top);
          if (cell) onCellClick(cell.string, cell.fret);
        }}
      />
      {view.inlays.map((inlay) => {
        const space = layout.spaces[inlay.fret - 1];
        if (!space) return null;
        const cx = layout.mirror(space.left + space.width / 2);
        return inlay.double ? (
          <g key={`inlay-${inlay.fret.toString()}`} className="inlay" data-testid="inlay">
            <circle cx={cx} cy={middleY - STRING_GAP} r={5} />
            <circle cx={cx} cy={middleY + STRING_GAP} r={5} />
          </g>
        ) : (
          <circle
            key={`inlay-${inlay.fret.toString()}`}
            className="inlay"
            data-testid="inlay"
            cx={cx}
            cy={middleY}
            r={5}
          />
        );
      })}
      {view.dimmed_below > 0 &&
        layout.spaces
          .filter((s) => s.fret <= view.dimmed_below)
          .map((s) => (
            <rect
              key={`dim-${s.fret.toString()}`}
              className="dimmed"
              data-testid="dimmed"
              x={Math.min(layout.mirror(s.left), layout.mirror(s.left + s.width))}
              y={VERTICAL_PADDING - 14}
              width={s.width}
              height={STRING_GAP * 5 + 28}
            />
          ))}
      {Array.from({ length: view.fret_count + 1 }, (_, n) => (
        <line
          key={`wire-${n.toString()}`}
          className={n === 0 ? 'nut' : 'fret-wire'}
          data-testid={n === 0 ? 'nut' : 'fret-wire'}
          x1={layout.wireX(n)}
          x2={layout.wireX(n)}
          y1={VERTICAL_PADDING - 14}
          y2={neckBottom - VERTICAL_PADDING + 14 + 8}
        />
      ))}
      {view.capo > 0 && (
        <rect
          className="capo"
          data-testid="capo-bar"
          x={layout.wireX(view.capo) - 5}
          y={VERTICAL_PADDING - 18}
          width={10}
          height={STRING_GAP * 5 + 36}
          rx={4}
        />
      )}
      {Array.from({ length: 6 }, (_, i) => 6 - i).map((string) => (
        <line
          key={`string-${string.toString()}`}
          className="string"
          data-testid="string"
          x1={layout.mirror(LABEL_MARGIN)}
          x2={layout.mirror(layout.width - 12)}
          y1={layout.yOfString(string)}
          y2={layout.yOfString(string)}
          strokeWidth={1 + (6 - string) * 0.35}
        />
      ))}
      {view.strings.map((s) => (
        <text
          key={`label-${s.string.toString()}`}
          className="string-label"
          data-testid="string-label"
          x={layout.labelX}
          y={layout.yOfString(s.string)}
          textAnchor="middle"
          dominantBaseline="central"
          fontSize={14.5}
        >
          {formatNote(s.note)}
        </text>
      ))}
      {view.barres.map((b) => {
        const x = layout.xOfFret(b.fret, false);
        const y1 = layout.yOfString(b.from);
        const y2 = layout.yOfString(b.to);
        return (
          <rect
            key={`barre-${b.fret.toString()}-${b.from.toString()}`}
            className="barre"
            data-testid="barre"
            x={x - 7}
            y={Math.min(y1, y2) - 9}
            width={14}
            height={Math.abs(y2 - y1) + 18}
            rx={7}
          />
        );
      })}
      {view.markers.map((m) => (
        <text
          key={`muted-${m.string.toString()}`}
          className="muted-marker"
          data-testid="muted-marker"
          x={layout.xOfFret(view.capo, true)}
          y={layout.yOfString(m.string)}
          textAnchor="middle"
          dominantBaseline="central"
          fontSize={18}
          aria-label={`String ${m.string.toString()} not played`}
          role="img"
        >
          ×
        </text>
      ))}
      <g className="positions">
        {placed.map(({ p, key, x, y }, i) => (
          <g
            key={key}
            ref={(el) => {
              if (el) refs.current.set(key, el);
              else refs.current.delete(key);
            }}
            role="button"
            tabIndex={i === 0 ? 0 : -1}
            aria-label={p.a11y_label}
            className="position"
            data-testid="position"
            data-string={p.string}
            data-fret={p.fret}
            onClick={() => {
              onPlayNote(p.string, p.fret);
            }}
            onKeyDown={(e) => {
              onKeyDown(e, key);
            }}
            onPointerEnter={() => {
              setHover(key);
            }}
            onPointerLeave={() => {
              setHover((h) => (h === key ? null : h));
            }}
            onFocus={() => {
              setHover(key);
            }}
            onBlur={() => {
              setHover((h) => (h === key ? null : h));
            }}
          >
            <NoteMarker
              role={p.role}
              x={x}
              y={y}
              radius={radius}
              label={labelFor(p, view.labels_mode)}
              fontSize={fontSize}
              order={p.order}
            />
          </g>
        ))}
      </g>
      {showFretNumbers &&
        view.fret_numbers.map((n) => {
          const space = layout.spaces[n - 1];
          if (!space) return null;
          return (
            <text
              key={`num-${n.toString()}`}
              className="fret-number"
              data-testid="fret-number"
              x={layout.mirror(space.left + space.width / 2)}
              y={layout.height - NUMBERS_HEIGHT / 2 + 4}
              textAnchor="middle"
              fontSize={14.5}
            >
              {n}
            </text>
          );
        })}
      {hovered && (
        <Tooltip
          position={hovered.p}
          x={hovered.x}
          y={hovered.y}
          width={layout.width}
          radius={radius}
        />
      )}
    </svg>
  );
}

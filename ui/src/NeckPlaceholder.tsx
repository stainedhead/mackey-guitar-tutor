const STRING_COUNT = 6;
const WIDTH = 1000;
const HEIGHT = 200;

/** An empty neck: six strings and a nut. The real fretboard is drawn from Rust's FretboardView. */
export function NeckPlaceholder() {
  const gap = HEIGHT / (STRING_COUNT + 1);
  return (
    <svg
      className="neck"
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      role="img"
      aria-label="Fretboard (placeholder, no notes yet)"
    >
      <rect className="neck-wood" x={0} y={0} width={WIDTH} height={HEIGHT} rx={8} />
      <line className="nut" x1={40} y1={gap / 2} x2={40} y2={HEIGHT - gap / 2} />
      {Array.from({ length: STRING_COUNT }, (_, i) => (
        <line
          key={i}
          className="string"
          data-testid="string"
          x1={40}
          y1={gap * (i + 1)}
          x2={WIDTH - 20}
          y2={gap * (i + 1)}
        />
      ))}
    </svg>
  );
}

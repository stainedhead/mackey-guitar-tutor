interface Props {
  /** Cents from the target, -50 to +50 (clamped for drawing). */
  cents: number | null;
  inTune: boolean;
}

/** The chromatic needle: -50 to +50 cents with a centre mark. The number and words say the same. */
export function Needle({ cents, inTune }: Props) {
  const clamped = cents === null ? 0 : Math.max(-50, Math.min(50, cents));
  const angle = (clamped / 50) * 60;
  return (
    <svg
      className={inTune ? 'needle in-tune' : 'needle'}
      viewBox="-120 -20 240 130"
      role="img"
      aria-label={
        cents === null ? 'Needle at rest' : `Needle at ${Math.round(cents).toString()} cents`
      }
    >
      <path className="needle-arc" d="M -104 90 A 120 120 0 0 1 104 90" fill="none" />
      {[-50, -25, 0, 25, 50].map((c) => {
        const a = (c / 50) * 60 * (Math.PI / 180);
        const x1 = Math.sin(a) * 96;
        const y1 = 90 - Math.cos(a) * 96;
        const x2 = Math.sin(a) * 108;
        const y2 = 90 - Math.cos(a) * 108;
        return (
          <line
            key={c}
            className={c === 0 ? 'needle-tick centre' : 'needle-tick'}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
          />
        );
      })}
      {[-50, 0, 50].map((c) => (
        <text
          key={c}
          className="needle-scale"
          x={Math.sin((c / 50) * 60 * (Math.PI / 180)) * 122}
          y={90 - Math.cos((c / 50) * 60 * (Math.PI / 180)) * 122 + 4}
          textAnchor="middle"
          fontSize={12}
        >
          {c > 0 ? `+${c.toString()}` : c.toString()}
        </text>
      ))}
      <g
        className="needle-hand"
        style={{ transform: `rotate(${angle.toString()}deg)`, transformOrigin: '0px 90px' }}
      >
        <line x1={0} y1={90} x2={0} y2={-6} />
        <circle cx={0} cy={90} r={5} />
      </g>
    </svg>
  );
}

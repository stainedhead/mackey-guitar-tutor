import { useId } from 'react';
import type { ChordFieldStatus } from '../bindings/ChordFieldStatus';

interface Props {
  text: string;
  status: ChordFieldStatus;
  onChange: (text: string) => void;
  onSubmit: (text: string) => void;
}

/** The chord field (FB-4.3, FB-4.4): a symbol or Roman numeral; errors show inline, not by colour alone. */
export function ChordField({ text, status, onChange, onSubmit }: Props) {
  const id = useId();
  const errorId = `${id}-error`;
  const error = status.kind === 'Error' ? status : null;
  return (
    <form
      className="chord-field"
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit(text);
      }}
    >
      <label htmlFor={id}>Chord</label>
      <input
        id={id}
        type="text"
        value={text}
        placeholder="Am7, D/F#, vi"
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-invalid={error !== null}
        aria-describedby={error ? errorId : undefined}
        className={error ? 'invalid' : undefined}
        onChange={(e) => {
          onChange(e.target.value);
        }}
      />
      <button type="submit">Show chord</button>
      {error && (
        <p id={errorId} className="field-error" role="alert">
          <span aria-hidden="true">⚠ </span>
          {error.message}
          {error.suggestion !== null && (
            <button
              type="button"
              className="link"
              onClick={() => {
                onSubmit(error.suggestion ?? '');
              }}
            >
              Use {error.suggestion}
            </button>
          )}
        </p>
      )}
    </form>
  );
}

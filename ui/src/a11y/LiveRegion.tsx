/** The screen-reader live region: cycling announcements land here (AC-FB-9.1c). */
export function LiveRegion({ message }: { message: string }) {
  return (
    <div
      className="sr-only"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      data-testid="live-region"
    >
      {message}
    </div>
  );
}

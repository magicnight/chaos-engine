export default function Loading() {
  return (
    <div className="flex flex-col items-center justify-center min-h-[40vh] gap-3">
      <div className="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" />
      <p className="text-xs text-[var(--muted)] font-mono tracking-wider">LOADING</p>
    </div>
  );
}

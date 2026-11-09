/**
 * Pointer drag helper for resize handles: reports the pointer position on every move and
 * releases capture on up. Keeps text from being selected while dragging.
 */
export function startDrag(e: PointerEvent, onMove: (ev: PointerEvent) => void, onEnd?: () => void) {
  if (e.button !== 0) return;
  e.preventDefault();
  const target = e.currentTarget as HTMLElement;
  target.setPointerCapture(e.pointerId);
  const prev = document.body.style.cursor;
  document.body.style.cursor = getComputedStyle(target).cursor;
  const move = (ev: PointerEvent) => onMove(ev);
  const up = () => {
    target.removeEventListener("pointermove", move);
    target.removeEventListener("pointerup", up);
    target.removeEventListener("pointercancel", up);
    document.body.style.cursor = prev;
    onEnd?.();
  };
  target.addEventListener("pointermove", move);
  target.addEventListener("pointerup", up);
  target.addEventListener("pointercancel", up);
}

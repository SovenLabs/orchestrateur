export type BlackholeLayout = {
  cx: number;
  cy: number;
  baseRadius: number;
  chatFade: number;
};

const ORB_INSET = { right: 72, top: 72 } as const;

export function computeBlackholeLayout(
  width: number,
  height: number,
  dockT: number,
): BlackholeLayout {
  const t = Math.max(0, Math.min(1, dockT));
  const expandedCx = width * 0.5;
  const expandedCy = height * 0.5;
  const expandedR = Math.min(width, height) * 0.28;
  const dockedCx = width - ORB_INSET.right;
  const dockedCy = ORB_INSET.top;
  const dockedR = 44;

  return {
    cx: expandedCx + (dockedCx - expandedCx) * t,
    cy: expandedCy + (dockedCy - expandedCy) * t,
    baseRadius: expandedR + (dockedR - expandedR) * t,
    chatFade: 1 - t * 0.35,
  };
}
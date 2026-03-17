import { useState, useEffect, useMemo } from "react";
import { X } from "lucide-react";
import type { TaskGraph, GraphNode, GraphEdge } from "@/types";
import * as api from "@/tauri/commands";

interface TaskGraphViewerProps {
  issueId: number;
  identifier: string;
  projectId: number;
  onClose: () => void;
}

const STATE_COLORS: Record<string, { bg: string; border: string; text: string }> = {
  unstarted: { bg: "rgb(39 39 42)", border: "rgb(63 63 70)", text: "rgb(161 161 170)" },
  started: { bg: "rgb(30 58 138)", border: "rgb(59 130 246)", text: "rgb(147 197 253)" },
  blocked: { bg: "rgb(127 29 29)", border: "rgb(239 68 68)", text: "rgb(252 165 165)" },
  completed: { bg: "rgb(20 83 45)", border: "rgb(34 197 94)", text: "rgb(134 239 172)" },
  discarded: { bg: "rgb(39 39 42)", border: "rgb(82 82 91)", text: "rgb(113 113 122)" },
};

const STATE_BADGE_COLORS: Record<string, string> = {
  unstarted: "bg-zinc-700 text-zinc-300",
  started: "bg-blue-900 text-blue-300",
  blocked: "bg-red-900 text-red-300",
  completed: "bg-green-900 text-green-300",
  discarded: "bg-zinc-800 text-zinc-500",
};

const NODE_WIDTH = 200;
const NODE_HEIGHT = 72;
const HORIZONTAL_GAP = 60;
const VERTICAL_GAP = 40;

interface LayoutNode extends GraphNode {
  x: number;
  y: number;
  depth: number;
}

function computeLayout(nodes: GraphNode[], edges: GraphEdge[]): LayoutNode[] {
  if (nodes.length === 0) return [];

  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  const childrenMap = new Map<number, number[]>();
  const parentCount = new Map<number, number>();

  for (const n of nodes) {
    childrenMap.set(n.id, []);
    parentCount.set(n.id, 0);
  }

  for (const edge of edges) {
    if (nodeMap.has(edge.from) && nodeMap.has(edge.to)) {
      childrenMap.get(edge.from)!.push(edge.to);
      parentCount.set(edge.to, (parentCount.get(edge.to) || 0) + 1);
    }
  }

  // Find roots (nodes with no incoming edges)
  const roots = nodes.filter(n => (parentCount.get(n.id) || 0) === 0);
  if (roots.length === 0) {
    // Cycle: just pick first node as root
    roots.push(nodes[0]);
  }

  // BFS to assign depths
  const depthMap = new Map<number, number>();
  const queue: number[] = [];
  for (const root of roots) {
    depthMap.set(root.id, 0);
    queue.push(root.id);
  }

  while (queue.length > 0) {
    const current = queue.shift()!;
    const currentDepth = depthMap.get(current)!;
    for (const child of childrenMap.get(current) || []) {
      const existingDepth = depthMap.get(child);
      if (existingDepth === undefined || existingDepth < currentDepth + 1) {
        depthMap.set(child, currentDepth + 1);
        queue.push(child);
      }
    }
  }

  // Assign depth 0 for unvisited nodes
  for (const n of nodes) {
    if (!depthMap.has(n.id)) {
      depthMap.set(n.id, 0);
    }
  }

  // Group by depth
  const layers = new Map<number, GraphNode[]>();
  for (const n of nodes) {
    const d = depthMap.get(n.id)!;
    if (!layers.has(d)) layers.set(d, []);
    layers.get(d)!.push(n);
  }

  const sortedDepths = Array.from(layers.keys()).sort((a, b) => a - b);

  // Compute positions: top-down layout
  const layoutNodes: LayoutNode[] = [];
  for (const depth of sortedDepths) {
    const layer = layers.get(depth)!;
    const totalWidth = layer.length * NODE_WIDTH + (layer.length - 1) * HORIZONTAL_GAP;
    const startX = -totalWidth / 2;

    for (let i = 0; i < layer.length; i++) {
      const n = layer[i];
      layoutNodes.push({
        ...n,
        x: startX + i * (NODE_WIDTH + HORIZONTAL_GAP),
        y: depth * (NODE_HEIGHT + VERTICAL_GAP),
        depth,
      });
    }
  }

  return layoutNodes;
}

export function TaskGraphViewer({ issueId, identifier, projectId, onClose }: TaskGraphViewerProps) {
  const [graph, setGraph] = useState<TaskGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    api.taskGraph(identifier)
      .then(setGraph)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [identifier, projectId]);

  const layoutNodes = useMemo(() => {
    if (!graph) return [];
    return computeLayout(graph.nodes, graph.edges);
  }, [graph]);

  const { viewBox, svgEdges } = useMemo(() => {
    if (layoutNodes.length === 0) {
      return { viewBox: "0 0 400 300", svgEdges: [] };
    }

    const nodePositions = new Map(layoutNodes.map(n => [n.id, n]));

    const edges = (graph?.edges || [])
      .map(e => {
        const from = nodePositions.get(e.from);
        const to = nodePositions.get(e.to);
        if (!from || !to) return null;
        return {
          x1: from.x + NODE_WIDTH / 2,
          y1: from.y + NODE_HEIGHT,
          x2: to.x + NODE_WIDTH / 2,
          y2: to.y,
          type: e.type,
        };
      })
      .filter((e): e is NonNullable<typeof e> => e !== null);

    const padding = 40;
    const minX = Math.min(...layoutNodes.map(n => n.x)) - padding;
    const minY = Math.min(...layoutNodes.map(n => n.y)) - padding;
    const maxX = Math.max(...layoutNodes.map(n => n.x + NODE_WIDTH)) + padding;
    const maxY = Math.max(...layoutNodes.map(n => n.y + NODE_HEIGHT)) + padding;

    return {
      viewBox: `${minX} ${minY} ${maxX - minX} ${maxY - minY}`,
      svgEdges: edges,
    };
  }, [layoutNodes, graph]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="relative flex max-h-[80vh] w-full max-w-4xl flex-col rounded-lg border border-zinc-700 bg-zinc-950 shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-800 px-5 py-3">
          <h2 className="text-sm font-semibold text-zinc-100">Dependency Graph</h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-4">
          {loading && (
            <div className="flex h-64 items-center justify-center text-sm text-zinc-500">
              Loading graph...
            </div>
          )}

          {error && (
            <div className="flex h-64 items-center justify-center text-sm text-red-400">
              Failed to load graph: {error}
            </div>
          )}

          {!loading && !error && layoutNodes.length === 0 && (
            <div className="flex h-64 items-center justify-center text-sm text-zinc-500">
              No dependencies found for this issue.
            </div>
          )}

          {!loading && !error && layoutNodes.length > 0 && (
            <svg
              viewBox={viewBox}
              className="h-full min-h-[300px] w-full"
              xmlns="http://www.w3.org/2000/svg"
            >
              <defs>
                <marker
                  id="arrowhead"
                  markerWidth="8"
                  markerHeight="6"
                  refX="8"
                  refY="3"
                  orient="auto"
                >
                  <polygon points="0 0, 8 3, 0 6" fill="rgb(113 113 122)" />
                </marker>
              </defs>

              {/* Edges */}
              {svgEdges.map((e, i) => {
                const midY = (e.y1 + e.y2) / 2;
                return (
                  <path
                    key={i}
                    d={`M ${e.x1} ${e.y1} C ${e.x1} ${midY}, ${e.x2} ${midY}, ${e.x2} ${e.y2}`}
                    fill="none"
                    stroke={e.type === "parent" ? "rgb(82 82 91)" : "rgb(113 113 122)"}
                    strokeWidth={e.type === "parent" ? 1 : 1.5}
                    strokeDasharray={e.type === "parent" ? "4 3" : "none"}
                    markerEnd="url(#arrowhead)"
                  />
                );
              })}

              {/* Nodes */}
              {layoutNodes.map(node => {
                const colors = STATE_COLORS[node.state] || STATE_COLORS.unstarted;
                const isRoot = node.id === issueId;
                return (
                  <g key={node.id}>
                    <rect
                      x={node.x}
                      y={node.y}
                      width={NODE_WIDTH}
                      height={NODE_HEIGHT}
                      rx={6}
                      fill={colors.bg}
                      stroke={isRoot ? "rgb(147 51 234)" : colors.border}
                      strokeWidth={isRoot ? 2 : 1}
                    />
                    {/* Identifier */}
                    <text
                      x={node.x + 10}
                      y={node.y + 20}
                      fill={colors.text}
                      fontSize={11}
                      fontFamily="monospace"
                      fontWeight={600}
                    >
                      {node.identifier}
                    </text>
                    {/* Title (truncated) */}
                    <text
                      x={node.x + 10}
                      y={node.y + 38}
                      fill="rgb(212 212 216)"
                      fontSize={11}
                    >
                      {node.title.length > 22 ? node.title.slice(0, 22) + "..." : node.title}
                    </text>
                    {/* State badge */}
                    <rect
                      x={node.x + 10}
                      y={node.y + 48}
                      width={node.type ? Math.min(node.type.length * 6.5 + 12, NODE_WIDTH - 20) : 60}
                      height={16}
                      rx={3}
                      fill={colors.border}
                      opacity={0.5}
                    />
                    <text
                      x={node.x + 16}
                      y={node.y + 60}
                      fill={colors.text}
                      fontSize={9}
                      fontWeight={500}
                    >
                      {node.type || node.state}
                    </text>
                  </g>
                );
              })}
            </svg>
          )}
        </div>

        {/* Legend */}
        {!loading && !error && layoutNodes.length > 0 && (
          <div className="flex items-center gap-4 border-t border-zinc-800 px-5 py-2.5">
            <span className="text-[10px] font-medium uppercase tracking-wider text-zinc-500">Legend</span>
            {Object.entries(STATE_BADGE_COLORS).map(([state, cls]) => (
              <span key={state} className={`rounded px-1.5 py-0.5 text-[10px] font-medium ${cls}`}>
                {state}
              </span>
            ))}
            <span className="ml-2 flex items-center gap-1.5 text-[10px] text-zinc-500">
              <svg width="20" height="8"><line x1="0" y1="4" x2="20" y2="4" stroke="rgb(113 113 122)" strokeWidth="1.5" /></svg>
              blocks
            </span>
            <span className="flex items-center gap-1.5 text-[10px] text-zinc-500">
              <svg width="20" height="8"><line x1="0" y1="4" x2="20" y2="4" stroke="rgb(82 82 91)" strokeWidth="1" strokeDasharray="4 3" /></svg>
              parent
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { X } from "lucide-react";
import type { DependencyGraph as DependencyGraphType, DependencyNode, DependencyEdge } from "@/types";
import * as api from "@/tauri/commands";

interface DependencyGraphProps {
  projectId: number;
  focusIssueId?: number;
  onClose: () => void;
  onClickIssue?: (issueId: number) => void;
}

interface LayoutNode extends DependencyNode {
  x: number;
  y: number;
  layer: number;
}

const NODE_WIDTH = 180;
const NODE_HEIGHT = 56;
const LAYER_SPACING = 220;
const NODE_SPACING = 80;

const STATUS_COLORS: Record<string, string> = {
  completed: "#22c55e",
  started: "#3b82f6",
  unstarted: "#94a3b8",
  blocked: "#ef4444",
  discarded: "#6b7280",
};

const PRIORITY_BORDER_COLORS: Record<string, string> = {
  urgent: "#ef4444",
  high: "#f97316",
  medium: "#eab308",
  low: "#3b82f6",
  none: "#6b7280",
};

function buildLayeredLayout(
  nodes: DependencyNode[],
  edges: DependencyEdge[],
  focusId?: number
): { layoutNodes: LayoutNode[]; layoutEdges: DependencyEdge[] } {
  if (nodes.length === 0) return { layoutNodes: [], layoutEdges: edges };

  // Build adjacency for topological sort (blocks edges only for layering)
  const adj = new Map<number, number[]>();
  const inDeg = new Map<number, number>();
  const nodeSet = new Set(nodes.map(n => n.id));

  for (const n of nodes) {
    adj.set(n.id, []);
    inDeg.set(n.id, 0);
  }

  for (const e of edges) {
    if (!nodeSet.has(e.source_id) || !nodeSet.has(e.target_id)) continue;
    if (e.relation_type === "blocks" || e.relation_type === "blocked_by") {
      const from = e.relation_type === "blocks" ? e.source_id : e.target_id;
      const to = e.relation_type === "blocks" ? e.target_id : e.source_id;
      adj.get(from)?.push(to);
      inDeg.set(to, (inDeg.get(to) || 0) + 1);
    }
  }

  // Topological sort via Kahn's algorithm to assign layers
  const layers = new Map<number, number>();
  const queue: number[] = [];

  for (const n of nodes) {
    if ((inDeg.get(n.id) || 0) === 0) {
      queue.push(n.id);
      layers.set(n.id, 0);
    }
  }

  while (queue.length > 0) {
    const curr = queue.shift()!;
    const currLayer = layers.get(curr) || 0;
    for (const next of adj.get(curr) || []) {
      const newLayer = currLayer + 1;
      if (!layers.has(next) || newLayer > layers.get(next)!) {
        layers.set(next, newLayer);
      }
      inDeg.set(next, (inDeg.get(next) || 0) - 1);
      if (inDeg.get(next) === 0) {
        queue.push(next);
      }
    }
  }

  // Assign layer 0 to any remaining (cycle or disconnected)
  for (const n of nodes) {
    if (!layers.has(n.id)) layers.set(n.id, 0);
  }

  // Group by layer
  const layerGroups = new Map<number, DependencyNode[]>();
  for (const n of nodes) {
    const l = layers.get(n.id) || 0;
    if (!layerGroups.has(l)) layerGroups.set(l, []);
    layerGroups.get(l)!.push(n);
  }

  // Position nodes
  const layoutNodes: LayoutNode[] = [];
  for (const [layer, group] of layerGroups) {
    const totalHeight = group.length * (NODE_HEIGHT + NODE_SPACING) - NODE_SPACING;
    const startY = -totalHeight / 2;
    group.forEach((node, idx) => {
      layoutNodes.push({
        ...node,
        x: layer * LAYER_SPACING,
        y: startY + idx * (NODE_HEIGHT + NODE_SPACING),
        layer,
      });
    });
  }

  // Center on focus node if provided
  if (focusId) {
    const focusNode = layoutNodes.find(n => n.id === focusId);
    if (focusNode) {
      const offsetX = -focusNode.x;
      const offsetY = -focusNode.y;
      for (const n of layoutNodes) {
        n.x += offsetX;
        n.y += offsetY;
      }
    }
  }

  return { layoutNodes, layoutEdges: edges };
}

export function DependencyGraph({ projectId, focusIssueId, onClose, onClickIssue }: DependencyGraphProps) {
  const [graph, setGraph] = useState<DependencyGraphType | null>(null);
  const [loading, setLoading] = useState(true);
  const [hoveredNode, setHoveredNode] = useState<number | null>(null);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [dragging, setDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    setLoading(true);
    api.dependencyGraph(projectId).then(g => {
      setGraph(g);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [projectId]);

  const { layoutNodes, layoutEdges } = useMemo(() => {
    if (!graph) return { layoutNodes: [], layoutEdges: [] };
    return buildLayeredLayout(graph.nodes, graph.edges, focusIssueId);
  }, [graph, focusIssueId]);

  // Auto-center
  useEffect(() => {
    if (layoutNodes.length > 0 && svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      const minX = Math.min(...layoutNodes.map(n => n.x));
      const maxX = Math.max(...layoutNodes.map(n => n.x + NODE_WIDTH));
      const minY = Math.min(...layoutNodes.map(n => n.y));
      const maxY = Math.max(...layoutNodes.map(n => n.y + NODE_HEIGHT));
      const graphW = maxX - minX + 100;
      const graphH = maxY - minY + 100;
      const fitZoom = Math.min(rect.width / graphW, rect.height / graphH, 1.2);
      setZoom(Math.max(0.3, fitZoom));
      setPan({
        x: rect.width / 2 - ((minX + maxX) / 2) * fitZoom,
        y: rect.height / 2 - ((minY + maxY) / 2) * fitZoom,
      });
    }
  }, [layoutNodes]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(3, zoom * delta));
    // Zoom toward cursor
    const rect = svgRef.current?.getBoundingClientRect();
    if (rect) {
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      setPan({
        x: mx - (mx - pan.x) * (newZoom / zoom),
        y: my - (my - pan.y) * (newZoom / zoom),
      });
    }
    setZoom(newZoom);
  }, [zoom, pan]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === svgRef.current || (e.target as SVGElement).tagName === "rect" && (e.target as SVGElement).getAttribute("data-bg") === "true") {
      setDragging(true);
      setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    }
  }, [pan]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (dragging) {
      setPan({ x: e.clientX - dragStart.x, y: e.clientY - dragStart.y });
    }
  }, [dragging, dragStart]);

  const handleMouseUp = useCallback(() => {
    setDragging(false);
  }, []);

  const getNodePos = useCallback((nodeId: number) => {
    const n = layoutNodes.find(nn => nn.id === nodeId);
    return n ? { x: n.x + NODE_WIDTH / 2, y: n.y + NODE_HEIGHT / 2 } : { x: 0, y: 0 };
  }, [layoutNodes]);

  if (loading) {
    return (
      <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center">
        <div className="bg-card rounded-xl p-8 text-center">
          <div className="text-sm text-muted-foreground">Loading dependency graph...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/80 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-3 bg-card/95 backdrop-blur border-b border-border">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-semibold">Dependency Graph</h2>
          <span className="text-xs text-muted-foreground">
            {layoutNodes.length} issues, {layoutEdges.length} relations
          </span>
        </div>
        <div className="flex items-center gap-3">
          {/* Legend */}
          <div className="flex items-center gap-3 text-[10px] text-muted-foreground">
            {Object.entries(STATUS_COLORS).map(([cat, color]) => (
              <div key={cat} className="flex items-center gap-1">
                <span className="h-2.5 w-2.5 rounded-sm" style={{ backgroundColor: color }} />
                {cat}
              </div>
            ))}
          </div>
          <div className="text-[10px] text-muted-foreground px-2 py-1 bg-muted rounded">
            Zoom: {Math.round(zoom * 100)}%
          </div>
          <button onClick={onClose} className="rounded-md p-1.5 hover:bg-muted transition-colors">
            <X className="h-4 w-4 text-muted-foreground" />
          </button>
        </div>
      </div>

      {/* Graph */}
      <div className="flex-1 overflow-hidden bg-background/95">
        {layoutNodes.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            No issues with dependency relations found in this project
          </div>
        ) : (
          <svg
            ref={svgRef}
            className="h-full w-full"
            style={{ cursor: dragging ? "grabbing" : "grab" }}
            onWheel={handleWheel}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
          >
            {/* Background */}
            <rect data-bg="true" x="0" y="0" width="100%" height="100%" fill="transparent" />

            <g transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}>
              {/* Edges */}
              <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
                  <polygon points="0 0, 10 3.5, 0 7" fill="#6b7280" />
                </marker>
                <marker id="arrowhead-blocks" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
                  <polygon points="0 0, 10 3.5, 0 7" fill="#ef4444" />
                </marker>
                <marker id="arrowhead-related" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
                  <polygon points="0 0, 10 3.5, 0 7" fill="#6366f1" />
                </marker>
              </defs>
              {layoutEdges.map((edge, i) => {
                const from = getNodePos(edge.source_id);
                const to = getNodePos(edge.target_id);
                const isBlocks = edge.relation_type === "blocks" || edge.relation_type === "blocked_by";
                const color = isBlocks ? "#ef4444" : edge.relation_type === "duplicate" ? "#f59e0b" : "#6366f1";
                const markerId = isBlocks ? "arrowhead-blocks" : "arrowhead-related";
                // Calculate path that avoids overlap with nodes
                const dx = to.x - from.x;
                const dy = to.y - from.y;
                const dist = Math.sqrt(dx * dx + dy * dy);
                if (dist === 0) return null;
                const startX = from.x + (dx / dist) * (NODE_WIDTH / 2);
                const startY = from.y + (dy / dist) * (NODE_HEIGHT / 2);
                const endX = to.x - (dx / dist) * (NODE_WIDTH / 2 + 12);
                const endY = to.y - (dy / dist) * (NODE_HEIGHT / 2 + 4);
                // Bezier control points for smooth curve
                const midX = (startX + endX) / 2;

                return (
                  <g key={i}>
                    <path
                      d={`M ${startX} ${startY} C ${midX} ${startY}, ${midX} ${endY}, ${endX} ${endY}`}
                      fill="none"
                      stroke={color}
                      strokeWidth={1.5}
                      strokeDasharray={isBlocks ? undefined : "6,4"}
                      opacity={0.7}
                      markerEnd={`url(#${markerId})`}
                    />
                  </g>
                );
              })}

              {/* Nodes */}
              {layoutNodes.map(node => {
                const fillColor = STATUS_COLORS[node.status_category] || "#94a3b8";
                const borderColor = PRIORITY_BORDER_COLORS[node.priority] || "#6b7280";
                const isHovered = hoveredNode === node.id;
                const isFocused = focusIssueId === node.id;

                return (
                  <g
                    key={node.id}
                    transform={`translate(${node.x}, ${node.y})`}
                    onMouseEnter={() => setHoveredNode(node.id)}
                    onMouseLeave={() => setHoveredNode(null)}
                    onClick={(e) => { e.stopPropagation(); onClickIssue?.(node.id); }}
                    style={{ cursor: "pointer" }}
                  >
                    {/* Node background */}
                    <rect
                      width={NODE_WIDTH}
                      height={NODE_HEIGHT}
                      rx={8}
                      ry={8}
                      fill={isHovered ? "var(--color-accent, #1e293b)" : "var(--color-card, #0f172a)"}
                      stroke={isFocused ? "#fbbf24" : borderColor}
                      strokeWidth={isFocused ? 2.5 : 1.5}
                      opacity={0.95}
                    />
                    {/* Status indicator */}
                    <rect
                      x={0}
                      y={0}
                      width={4}
                      height={NODE_HEIGHT}
                      rx={4}
                      fill={fillColor}
                    />
                    {/* Identifier */}
                    <text
                      x={14}
                      y={20}
                      fontSize={10}
                      fontFamily="monospace"
                      fill="var(--color-muted-foreground, #94a3b8)"
                      opacity={0.7}
                    >
                      {node.identifier}
                    </text>
                    {/* Title */}
                    <text
                      x={14}
                      y={38}
                      fontSize={12}
                      fontWeight={500}
                      fill="var(--color-foreground, #e2e8f0)"
                    >
                      {node.title.length > 20 ? node.title.slice(0, 19) + "..." : node.title}
                    </text>
                    {/* Priority dot */}
                    <circle
                      cx={NODE_WIDTH - 14}
                      cy={14}
                      r={4}
                      fill={PRIORITY_BORDER_COLORS[node.priority] || "#6b7280"}
                    />

                    {/* Tooltip */}
                    {isHovered && (
                      <g>
                        <rect
                          x={0}
                          y={NODE_HEIGHT + 6}
                          width={Math.max(200, node.title.length * 7 + 20)}
                          height={52}
                          rx={6}
                          fill="var(--color-popover, #1e293b)"
                          stroke="var(--color-border, #334155)"
                          strokeWidth={1}
                        />
                        <text x={8} y={NODE_HEIGHT + 22} fontSize={11} fontWeight={600} fill="var(--color-foreground, #e2e8f0)">
                          {node.title}
                        </text>
                        <text x={8} y={NODE_HEIGHT + 38} fontSize={10} fill="var(--color-muted-foreground, #94a3b8)">
                          {node.assignee_name ? `Assigned: ${node.assignee_name}` : "Unassigned"} | {node.priority} priority
                        </text>
                        <text x={8} y={NODE_HEIGHT + 52} fontSize={10} fill="var(--color-muted-foreground, #94a3b8)">
                          Status: {node.status_category}
                        </text>
                      </g>
                    )}
                  </g>
                );
              })}
            </g>
          </svg>
        )}
      </div>
    </div>
  );
}

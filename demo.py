import matplotlib.pyplot as plt
import numpy as np

plt.rcParams.update({
    "font.family": "serif",
    "font.size": 11,
    "axes.labelsize": 11,
    "xtick.labelsize": 10,
    "ytick.labelsize": 10,
    "figure.dpi": 300,
    "axes.linewidth": 0.9,
})

# -----------------------------
# Data
# -----------------------------
T = [0, 1, 2, 3, 4, 5]
me_T = [18.46, 17.71, 17.08, 16.83, 16.72, 16.78]
time_T = [68.4, 71.0, 73.6, 76.0, 80.7, 85.9]

k = [8, 10, 12, 14, 16, 18]
me_k = [17.41, 17.03, 16.83, 16.79, 16.80, 16.94]
time_k = [72.8, 74.2, 76.0, 78.8, 81.5, 84.9]

# -----------------------------
# Color palettes
# -----------------------------
# Figure 1: blue-violet
line1 = (124/255, 145/255, 188/255)
accent1 = (102/255, 91/255, 157/255)
fill1 = (238/255, 243/255, 251/255)

# Figure 2: pink-lilac
line2 = (194/255, 129/255, 163/255)
accent2 = (132/255, 89/255, 156/255)
fill2 = (249/255, 225/255, 236/255)

grid_c = (0.88, 0.88, 0.90)

def style_ax(ax):
    ax.set_facecolor("white")
    ax.grid(True, linestyle='--', linewidth=0.7, alpha=0.55, color=grid_c)
    for spine in ax.spines.values():
        spine.set_alpha(0.72)

def add_focus_box(ax, x, y, text, edge_c, fill_c, dx_pts, dy_pts):
    ax.annotate(
        text,
        xy=(x, y),
        xytext=(dx_pts, dy_pts),
        textcoords='offset points',
        fontsize=9.2,
        color=edge_c,
        ha='left',
        va='center',
        bbox=dict(
            boxstyle="round,pad=0.30",
            fc=fill_c,
            ec=edge_c,
            lw=0.9
        ),
        arrowprops=dict(
            arrowstyle='-',
            lw=0.9,
            color=edge_c,
            shrinkA=4,
            shrinkB=4
        ),
        zorder=7
    )

def bbox_overlap(bb1, bb2, pad=3.0):
    return not (
        bb1.x1 + pad < bb2.x0 or
        bb1.x0 - pad > bb2.x1 or
        bb1.y1 + pad < bb2.y0 or
        bb1.y0 - pad > bb2.y1
    )

def point_in_rect(px, py, rect):
    x0, y0, x1, y1 = rect
    return x0 <= px <= x1 and y0 <= py <= y1

def seg_intersects_rect(p1, p2, rect):
    x0, y0, x1, y1 = rect
    minx, maxx = min(p1[0], p2[0]), max(p1[0], p2[0])
    miny, maxy = min(p1[1], p2[1]), max(p1[1], p2[1])
    if maxx < x0 or minx > x1 or maxy < y0 or miny > y1:
        return False

    if point_in_rect(p1[0], p1[1], rect) or point_in_rect(p2[0], p2[1], rect):
        return True

    def ccw(a, b, c):
        return (c[1] - a[1]) * (b[0] - a[0]) > (b[1] - a[1]) * (c[0] - a[0])

    def seg_intersect(a, b, c, d):
        return ccw(a, c, d) != ccw(b, c, d) and ccw(a, b, c) != ccw(a, b, d)

    corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    rect_edges = list(zip(corners, corners[1:] + corners[:1]))
    for c, d in rect_edges:
        if seg_intersect(p1, p2, c, d):
            return True
    return False

def label_hits_polyline(bb, poly_disp, pad=2.5):
    rect = (bb.x0 - pad, bb.y0 - pad, bb.x1 + pad, bb.y1 + pad)
    for i in range(len(poly_disp) - 1):
        if seg_intersects_rect(poly_disp[i], poly_disp[i + 1], rect):
            return True
    return False

def smart_label_points(ax, fig, xs, ys, labels, color, polyline_xy, candidates):
    """
    Place labels around points using candidate offsets in display space,
    avoiding the plotted polyline and already-placed labels.
    """
    fig.canvas.draw()
    renderer = fig.canvas.get_renderer()
    poly_disp = ax.transData.transform(np.column_stack(polyline_xy))
    placed = []

    for x, y, lab in zip(xs, ys, labels):
        ok = False
        for dx, dy in candidates:
            txt = ax.annotate(
                lab,
                xy=(x, y),
                xytext=(dx, dy),
                textcoords='offset points',
                fontsize=9,
                color=color,
                ha='left',
                va='center',
                zorder=6
            )
            fig.canvas.draw()
            bb = txt.get_window_extent(renderer=renderer)

            overlap = any(bbox_overlap(bb, oldbb, pad=2.0) for oldbb in placed)
            hit_line = label_hits_polyline(bb, poly_disp, pad=2.0)

            if overlap or hit_line:
                txt.remove()
            else:
                placed.append(bb)
                ok = True
                break

        if not ok:
            # last-resort fallback, still close to point
            txt = ax.annotate(
                lab,
                xy=(x, y),
                xytext=(8, 8),
                textcoords='offset points',
                fontsize=9,
                color=color,
                ha='left',
                va='center',
                zorder=6
            )
            fig.canvas.draw()
            placed.append(txt.get_window_extent(renderer=renderer))

# =========================================================
# Figure 1: T trade-off
# =========================================================
# fig, ax = plt.subplots(figsize=(6.0, 4.15))
# style_ax(ax)

# ax.plot(
#     time_T, me_T,
#     color=line1,
#     linewidth=2.2,
#     marker='o',
#     markersize=6.8,
#     markerfacecolor='white',
#     markeredgecolor=line1,
#     markeredgewidth=1.2,
#     zorder=3
# )
# ax.scatter(time_T, me_T, s=28, color=line1, alpha=0.18, zorder=2)

# # labels except T=3
# xs_T = [time_T[i] for i in range(len(T)) if T[i] != 3]
# ys_T = [me_T[i] for i in range(len(T)) if T[i] != 3]
# labels_T = [rf'$T={T[i]}$' for i in range(len(T)) if T[i] != 3]

# # 图1已基本满意，用稍宽候选
# candidates_T = [
#     (-18, 10), (8, 10), (-18, -10), (8, -10),
#     (-24, 0), (10, 0), (-4, 14), (-4, -14)
# ]
# smart_label_points(ax, fig, xs_T, ys_T, labels_T, accent1, (time_T, me_T), candidates_T)

# # focus box for T=3
# idx_T = T.index(3)
# ax.scatter(
#     time_T[idx_T], me_T[idx_T],
#     s=95, color=accent1, edgecolors='white', linewidth=1.0, zorder=5
# )
# add_focus_box(
#     ax,
#     time_T[idx_T], me_T[idx_T],
#     r'$T=3$' + '\n(76.0 ms, 16.83%)',
#     accent1, fill1,
#     dx_pts=12, dy_pts=22
# )

# ax.set_xlabel('GPU Time (ms)')
# ax.set_ylabel('ME (%)')
# ax.invert_yaxis()
# ax.set_xlim(66.5, 87.5)
# ax.set_ylim(18.8, 16.45)

# plt.tight_layout()
# plt.savefig('ablation_T_tradeoff_final.png', dpi=300, bbox_inches='tight')
# plt.show()

# =========================================================
# Figure 2: k trade-off
# =========================================================
fig, ax = plt.subplots(figsize=(6.0, 4.15))
style_ax(ax)

ax.plot(
    time_k, me_k,
    color=line2,
    linewidth=2.2,
    marker='s',
    markersize=6.6,
    markerfacecolor='white',
    markeredgecolor=line2,
    markeredgewidth=1.2,
    zorder=3
)
ax.scatter(time_k, me_k, s=28, color=line2, alpha=0.18, zorder=2)

# labels except k=12
xs_k = [time_k[i] for i in range(len(k)) if k[i] != 12]
ys_k = [me_k[i] for i in range(len(k)) if k[i] != 12]
labels_k = [rf'$k={k[i]}$' for i in range(len(k)) if k[i] != 12]

# 图2改为手动偏移：按 k 值设置每个标签的 (dx, dy)，单位为 points
manual_offsets_k = {
    8: (6, -10),
    10: (6, -10),
    14: (6, 10),
    16: (6, 10),
    18: (6,10),
}

for kk, x, y, lab in zip([v for v in k if v != 12], xs_k, ys_k, labels_k):
    dx, dy = manual_offsets_k.get(kk, (8, 8))
    ax.annotate(
        lab,
        xy=(x, y),
        xytext=(dx, dy),
        textcoords='offset points',
        fontsize=9,
        color=accent2,
        ha='left',
        va='center',
        zorder=6
    )

# focus box for k=12
idx_k = k.index(12)
ax.scatter(
    time_k[idx_k], me_k[idx_k],
    s=95, color=accent2, edgecolors='white', linewidth=1.0, zorder=5
)

# manually adjustable position for the focus box of k=12 (offset points)
focus_k_dx_pts = 12   # >0 right, <0 left
focus_k_dy_pts = -32  # >0 down, <0 up

add_focus_box(
    ax,
    time_k[idx_k], me_k[idx_k],
    r'$k=12$' + '\n(76.0 ms, 16.83%)',
    accent2, fill2,
    dx_pts=focus_k_dx_pts, dy_pts=focus_k_dy_pts
)

ax.set_xlabel('GPU Time (ms)')
ax.set_ylabel('ME (%)')
ax.invert_yaxis()
ax.set_xlim(71.0, 86.5)
ax.set_ylim(17.55, 16.68)

plt.tight_layout()
plt.savefig('ablation_k_tradeoff_final.png', dpi=300, bbox_inches='tight')
plt.show()
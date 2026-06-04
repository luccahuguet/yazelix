// Common cursor trail shader functions
// This file is included by the build script when generating cursor trail variants
// DO NOT use this file directly - it's not a complete shader

float getSdfRectangle(in vec2 p, in vec2 xy, in vec2 b)
{
    vec2 d = abs(p - xy) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

// Based on Inigo Quilez's 2D distance functions article: https://iquilezles.org/articles/distfunctions2d/
// Potentially optimized by eliminating conditionals and loops to enhance performance and reduce branching

float seg(in vec2 p, in vec2 a, in vec2 b, inout float s, float d) {
    vec2 e = b - a;
    vec2 w = p - a;
    vec2 proj = a + e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    float segd = dot(p - proj, p - proj);
    d = min(d, segd);

    float c0 = step(0.0, p.y - a.y);
    float c1 = 1.0 - step(0.0, p.y - b.y);
    float c2 = 1.0 - step(0.0, e.x * w.y - e.y * w.x);
    float allCond = c0 * c1 * c2;
    float noneCond = (1.0 - c0) * (1.0 - c1) * (1.0 - c2);
    float flip = mix(1.0, -1.0, step(0.5, allCond + noneCond));
    s *= flip;
    return d;
}

float getSdfParallelogram(in vec2 p, in vec2 v0, in vec2 v1, in vec2 v2, in vec2 v3) {
    float s = 1.0;
    float d = dot(p - v0, p - v0);

    d = seg(p, v0, v3, s, d);
    d = seg(p, v1, v0, s, d);
    d = seg(p, v2, v1, s, d);
    d = seg(p, v3, v2, s, d);

    return s * sqrt(d);
}

vec2 normalize(vec2 value, float isPosition) {
    return (value * 2.0 - (iResolution.xy * isPosition)) / iResolution.y;
}

float antialising(float distance) {
    return 1. - smoothstep(0., normalize(vec2(2., 2.), 0.).x, distance);
}

float determineStartVertexFactor(vec2 c, vec2 p) {
    float condition1 = step(p.x, c.x) * step(c.y, p.y);
    float condition2 = step(c.x, p.x) * step(p.y, c.y);
    return 1.0 - max(condition1, condition2);
}

vec2 getRectangleCenter(vec4 rectangle) {
    return vec2(rectangle.x + (rectangle.z / 2.), rectangle.y - (rectangle.w / 2.));
}

float ease(float x) {
    return pow(1.0 - x, 3.0);
}

vec4 saturate(vec4 color, float factor) {
    float gray = dot(color, vec4(0.299, 0.587, 0.114, 0.));
    return mix(vec4(gray), color, factor);
}

float yazelixGlowMask(float sdf, float offset, float width, float widthScale, float strength) {
    if (strength <= 0.0) {
        return 0.0;
    }

    return strength * (1.0 - smoothstep(offset, offset + (width * widthScale), sdf));
}

float trailGlowMask(float sdf, float offset, float width) {
    return yazelixGlowMask(sdf, offset, width, YAZELIX_TRAIL_GLOW_WIDTH_SCALE, YAZELIX_TRAIL_GLOW_STRENGTH);
}

float trailEdgeMask(float sdf, float offset, float width) {
    return 1.0 - smoothstep(0.0, width * YAZELIX_TRAIL_EDGE_WIDTH_SCALE, sdf + (offset * YAZELIX_TRAIL_EDGE_WIDTH_SCALE));
}

float cursorGlowMask(float sdf, float offset, float width) {
    return yazelixGlowMask(sdf, offset, width, YAZELIX_CURSOR_GLOW_WIDTH_SCALE, YAZELIX_CURSOR_GLOW_STRENGTH);
}

float cursorEdgeMask(float sdf, float offset, float width) {
    return 1.0 - smoothstep(0.0, width * YAZELIX_CURSOR_EDGE_WIDTH_SCALE, sdf + (offset * YAZELIX_CURSOR_EDGE_WIDTH_SCALE));
}

vec4 applyTrailLayer(vec4 base, vec4 overlay, float mask) {
    return mix(base, overlay, clamp(mask, 0.0, 1.0));
}

float trailCoreMask(float sdf, float offset) {
    return step(sdf + (offset * YAZELIX_TRAIL_CORE_OFFSET_SCALE), 0.0);
}

float yazelixRioTrailAnimatingFactor() {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    return iYazelixRioTrailAnimating == 0 ? 0.0 : 1.0;
#else
    return 0.0;
#endif
}

float yazelixRioTrailActiveFactor() {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    return iYazelixRioTrailActive == 0 ? 0.0 : 1.0;
#else
    return 0.0;
#endif
}

vec4 yazelixRioTrailAnimatedRect() {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    return vec4(normalize(iYazelixRioTrailAnimatedCursor.xy, 1.), normalize(iYazelixRioTrailAnimatedCursor.zw, 0.));
#else
    return vec4(0.0);
#endif
}

vec4 yazelixRioTrailCursorRect(vec4 fallback) {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    if (iYazelixRioTrailAnimating == 0) {
        return fallback;
    }

    return yazelixRioTrailAnimatedRect();
#else
    return fallback;
#endif
}

float yazelixRioTrailMotionFactor(vec4 currentCursor) {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    if (iYazelixRioTrailActive == 0) {
        return 0.0;
    }

    vec4 animatedCursor = yazelixRioTrailAnimatedRect();
    vec2 cursorSize = max(currentCursor.zw, vec2(0.0001));
    vec2 extraSize = max(animatedCursor.zw - currentCursor.zw, vec2(0.0));
    float stretch = clamp(length(extraSize / cursorSize) * 0.75, 0.0, 1.0);
    float recentMove = 1.0 - smoothstep(0.0, 0.10, max(iTime - iTimeCursorChange, 0.0));
    return max(max(yazelixRioTrailAnimatingFactor(), stretch), recentMove * 0.75);
#else
    return 0.0;
#endif
}

vec2 yazelixRioTrailCenter(vec2 fallback) {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    if (iYazelixRioTrailAnimating == 0) {
        return fallback;
    }

    return getRectangleCenter(yazelixRioTrailAnimatedRect());
#else
    return fallback;
#endif
}

float yazelixRioTrailCursorSdf(in vec2 vu, vec4 fallbackCursor, in vec2 offsetFactor) {
    vec4 cursor = yazelixRioTrailCursorRect(fallbackCursor);
    return getSdfRectangle(vu, cursor.xy - (cursor.zw * offsetFactor), cursor.zw * 0.5);
}

float yazelixRioAuraMask(vec2 point, vec2 center, vec2 cursorSize, float active, float spread) {
    float radius = max(length(cursorSize) * spread * YAZELIX_TRAIL_GLOW_WIDTH_SCALE, 0.032);
    float distanceToCenter = distance(point, center);
    return active * YAZELIX_TRAIL_GLOW_STRENGTH * (1.0 - smoothstep(radius * 0.20, radius, distanceToCenter));
}

float yazelixRioAuraCoreMask(vec2 point, vec2 center, vec2 cursorSize, float active, float spread) {
    float radius = max(length(cursorSize) * spread * YAZELIX_TRAIL_GLOW_WIDTH_SCALE, 0.032);
    float distanceToCenter = distance(point, center);
    return active * YAZELIX_TRAIL_GLOW_STRENGTH * (1.0 - smoothstep(radius * 0.06, radius * 0.34, distanceToCenter));
}

vec4 applyYazelixTerminalRioAura(vec4 color, vec2 point, vec2 center, vec2 cursorSize, vec4 outerColor, vec4 coreColor) {
    float motion = yazelixRioTrailMotionFactor(vec4(center - (cursorSize * 0.5), cursorSize));
    float spread = mix(0.45, 0.85, motion);
    float aura = yazelixRioAuraMask(point, center, cursorSize, motion, spread);
    float core = yazelixRioAuraCoreMask(point, center, cursorSize, motion, spread);
    color = applyTrailLayer(color, outerColor, aura * mix(0.12, 0.25, motion));
    color = applyTrailLayer(color, coreColor, core * mix(0.16, 0.32, motion));
    return color;
}

float yazelixRioTrailSdf(in vec2 vu, in vec2 offsetFactor) {
#if defined(YAZELIX_TERMINAL_RIO_TRAIL)
    vec4 animatedCursor = yazelixRioTrailAnimatedRect();
    float bboxSdf = getSdfRectangle(vu, animatedCursor.xy - (animatedCursor.zw * offsetFactor), animatedCursor.zw * 0.5);

    vec2 q0 = normalize(iYazelixRioTrailCorners[0].xy, 1.);
    vec2 q1 = normalize(iYazelixRioTrailCorners[1].xy, 1.);
    vec2 q2 = normalize(iYazelixRioTrailCorners[2].xy, 1.);
    vec2 q3 = normalize(iYazelixRioTrailCorners[3].xy, 1.);
    float quadSdf = getSdfParallelogram(vu, q0, q1, q2, q3);

    return min(quadSdf, bboxSdf);
#else
    return 0.0;
#endif
}

void renderMonoColorTrail(
    out vec4 fragColor,
    in vec2 fragCoord,
    vec4 trailColor,
    vec4 accentColor,
    float duration,
    float mod,
    float coreSaturation
) {
    #if !defined(WEB)
    fragColor = texture(iChannel0, fragCoord.xy / iResolution.xy);
    #endif
    vec2 vu = normalize(fragCoord, 1.);
    vec2 offsetFactor = vec2(-.5, 0.5);

    vec4 currentCursor = vec4(normalize(iCurrentCursor.xy, 1.), normalize(iCurrentCursor.zw, 0.));
    vec4 previousCursor = vec4(normalize(iPreviousCursor.xy, 1.), normalize(iPreviousCursor.zw, 0.));

    vec2 centerCC = getRectangleCenter(currentCursor);
    vec2 centerCP = getRectangleCenter(previousCursor);
    float vertexFactor = determineStartVertexFactor(currentCursor.xy, previousCursor.xy);
    float invertedVertexFactor = 1.0 - vertexFactor;

    vec2 v0 = vec2(currentCursor.x + currentCursor.z * vertexFactor, currentCursor.y - currentCursor.w);
    vec2 v1 = vec2(currentCursor.x + currentCursor.z * invertedVertexFactor, currentCursor.y);
    vec2 v2 = vec2(previousCursor.x + currentCursor.z * invertedVertexFactor, previousCursor.y);
    vec2 v3 = vec2(previousCursor.x + currentCursor.z * vertexFactor, previousCursor.y - previousCursor.w);

    vec4 animatedCursor = yazelixRioTrailCursorRect(currentCursor);
    float sdfCurrentCursor = yazelixRioTrailCursorSdf(vu, currentCursor, offsetFactor);
    float sdfTrail = getSdfParallelogram(vu, v0, v1, v2, v3);
    float rioTrailActive = yazelixRioTrailActiveFactor();
    float rioTrailAnimating = yazelixRioTrailAnimatingFactor();
    float rioTrailMotion = yazelixRioTrailMotionFactor(currentCursor);
    sdfTrail = mix(sdfTrail, yazelixRioTrailSdf(vu, offsetFactor), rioTrailAnimating);
    float trailGlowOffset = mix(mod + 0.010, -0.010, rioTrailAnimating);
    float trailGlowWidth = mix(0.035, mix(0.055, 0.090, rioTrailMotion), rioTrailAnimating);
    float trailGlowGain = mix(1.0, mix(0.80, 1.35, rioTrailMotion), rioTrailActive);
    float trailEdgeOffset = mix(mod, -0.004, rioTrailAnimating);
    float trailEdgeWidth = mix(0.006, mix(0.014, 0.026, rioTrailMotion), rioTrailAnimating);
    float trailCoreOffset = mix(mod, -0.002, rioTrailAnimating);
    float trailSaturation = mix(1.5, mix(1.75, 2.35, rioTrailMotion), rioTrailAnimating);
    float cursorGlowWidth = mix(0.004, mix(0.002, 0.014, rioTrailMotion), rioTrailActive);
    float cursorGlowGain = mix(1.0, mix(0.75, 1.45, rioTrailMotion), rioTrailActive);
    float cursorEdgeWidth = mix(0.004, mix(0.003, 0.012, rioTrailMotion), rioTrailActive);

    float progress = clamp((iTime - iTimeCursorChange) / duration, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    vec4 trail = fragColor;
    trail = applyTrailLayer(trail, saturate(accentColor, trailSaturation), trailGlowMask(sdfTrail, trailGlowOffset, trailGlowWidth) * trailGlowGain);
    trail = applyTrailLayer(trail, saturate(trailColor, trailSaturation), trailEdgeMask(sdfTrail, trailEdgeOffset, trailEdgeWidth));
    trail = mix(trail, saturate(trailColor, mix(coreSaturation, 1.8, rioTrailAnimating)), trailCoreMask(sdfTrail, trailCoreOffset));
    trail = applyTrailLayer(trail, saturate(accentColor, trailSaturation), cursorGlowMask(sdfCurrentCursor, .002, cursorGlowWidth) * cursorGlowGain);
    trail = applyTrailLayer(trail, saturate(trailColor, trailSaturation), cursorEdgeMask(sdfCurrentCursor, .002, cursorEdgeWidth));
    float revealMix = 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength);
    fragColor = mix(trail, fragColor, mix(revealMix, 0.0, rioTrailAnimating));
    fragColor = applyYazelixTerminalRioAura(
        fragColor,
        vu,
        getRectangleCenter(animatedCursor),
        animatedCursor.zw,
        saturate(accentColor, trailSaturation),
        saturate(trailColor, mix(coreSaturation, 1.8, rioTrailAnimating))
    );
}

void renderSplitColorTrail(
    out vec4 fragColor,
    in vec2 fragCoord,
    vec4 color0,
    vec4 color1,
    float duration,
    float horizontal,
    float blendEnabled
) {
    #if !defined(WEB)
    fragColor = texture(iChannel0, fragCoord.xy / iResolution.xy);
    #endif
    vec2 vu = normalize(fragCoord, 1.);
    vec2 offsetFactor = vec2(-.5, 0.5);

    vec4 currentCursor = vec4(normalize(iCurrentCursor.xy, 1.), normalize(iCurrentCursor.zw, 0.));
    vec4 previousCursor = vec4(normalize(iPreviousCursor.xy, 1.), normalize(iPreviousCursor.zw, 0.));

    vec2 centerCC = getRectangleCenter(currentCursor);
    vec2 centerCP = getRectangleCenter(previousCursor);
    float vertexFactor = determineStartVertexFactor(currentCursor.xy, previousCursor.xy);
    float invertedVertexFactor = 1.0 - vertexFactor;

    vec2 v0 = vec2(currentCursor.x + currentCursor.z * vertexFactor, currentCursor.y - currentCursor.w);
    vec2 v1 = vec2(currentCursor.x + currentCursor.z * invertedVertexFactor, currentCursor.y);
    vec2 v2 = vec2(previousCursor.x + currentCursor.z * invertedVertexFactor, previousCursor.y);
    vec2 v3 = vec2(previousCursor.x + currentCursor.z * vertexFactor, previousCursor.y - previousCursor.w);

    vec4 animatedCursor = yazelixRioTrailCursorRect(currentCursor);
    float sdfCurrentCursor = yazelixRioTrailCursorSdf(vu, currentCursor, offsetFactor);
    float sdfTrail = getSdfParallelogram(vu, v0, v1, v2, v3);
    float rioTrailActive = yazelixRioTrailActiveFactor();
    float rioTrailAnimating = yazelixRioTrailAnimatingFactor();
    float rioTrailMotion = yazelixRioTrailMotionFactor(currentCursor);
    sdfTrail = mix(sdfTrail, yazelixRioTrailSdf(vu, offsetFactor), rioTrailAnimating);

    float progress = clamp((iTime - iTimeCursorChange) / duration, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    float mod = .005;
    vec2 splitCenter = yazelixRioTrailCenter(centerCC);
    vec2 dir = normalize(vu - splitCenter + 1e-6);
    float splitAxis = mix(dir.x, dir.y, clamp(horizontal, 0.0, 1.0));
    float hardMix = step(0.0, splitAxis);
    float softMix = smoothstep(-0.08, 0.08, splitAxis);
    float splitMix = mix(hardMix, softMix, clamp(blendEnabled, 0.0, 1.0));
    float pulse = 0.05 * sin(iTime * 1.6) * clamp(blendEnabled, 0.0, 1.0);
    float edgeMix = clamp(splitMix + pulse * 0.45, 0.0, 1.0);
    float trailGlowOffset = mix(mod + 0.010, -0.010, rioTrailAnimating);
    float trailGlowWidth = mix(0.035, mix(0.055, 0.090, rioTrailMotion), rioTrailAnimating);
    float trailGlowGain = mix(1.0, mix(0.80, 1.35, rioTrailMotion), rioTrailActive);
    float trailEdgeOffset = mix(mod, -0.004, rioTrailAnimating);
    float trailEdgeWidth = mix(0.006, mix(0.014, 0.026, rioTrailMotion), rioTrailAnimating);
    float trailCoreOffset = mix(mod, -0.002, rioTrailAnimating);
    float trailSaturation = mix(1.45, mix(1.75, 2.35, rioTrailMotion), rioTrailAnimating);
    float cursorGlowWidth = mix(0.004, mix(0.002, 0.014, rioTrailMotion), rioTrailActive);
    float cursorGlowGain = mix(1.0, mix(0.75, 1.45, rioTrailMotion), rioTrailActive);
    float cursorEdgeWidth = mix(0.004, mix(0.003, 0.012, rioTrailMotion), rioTrailActive);

    vec4 base = mix(color0, color1, splitMix);
    vec4 edge = mix(color0, color1, edgeMix);

    vec4 trail = fragColor;
    trail = applyTrailLayer(trail, saturate(base, trailSaturation), trailGlowMask(sdfTrail, trailGlowOffset, trailGlowWidth) * trailGlowGain);
    trail = applyTrailLayer(trail, saturate(edge, mix(1.55, 2.25, rioTrailAnimating)), trailEdgeMask(sdfTrail, trailEdgeOffset, trailEdgeWidth));
    trail = mix(trail, saturate(base, trailSaturation), trailCoreMask(sdfTrail, trailCoreOffset));

    trail = applyTrailLayer(trail, saturate(edge, mix(1.55, 2.25, rioTrailAnimating)), cursorGlowMask(sdfCurrentCursor, .002, cursorGlowWidth) * cursorGlowGain);
    trail = applyTrailLayer(trail, saturate(base, trailSaturation), cursorEdgeMask(sdfCurrentCursor, .002, cursorEdgeWidth));
    float revealMix = 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength);
    fragColor = mix(trail, fragColor, mix(revealMix, 0.0, rioTrailAnimating));
    fragColor = applyYazelixTerminalRioAura(
        fragColor,
        vu,
        getRectangleCenter(animatedCursor),
        animatedCursor.zw,
        saturate(edge, mix(1.55, 2.25, rioTrailAnimating)),
        saturate(base, trailSaturation)
    );
}

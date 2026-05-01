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

void renderSimpleDualColorTrail(
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

    float sdfCurrentCursor = getSdfRectangle(vu, currentCursor.xy - (currentCursor.zw * offsetFactor), currentCursor.zw * 0.5);
    float sdfTrail = getSdfParallelogram(vu, v0, v1, v2, v3);

    float progress = clamp((iTime - iTimeCursorChange) / duration, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    vec4 trail = fragColor;
    trail = applyTrailLayer(trail, saturate(accentColor, 1.5), trailGlowMask(sdfTrail, mod + 0.010, 0.035));
    trail = applyTrailLayer(trail, saturate(trailColor, 1.5), trailEdgeMask(sdfTrail, mod, 0.006));
    trail = mix(trail, saturate(trailColor, coreSaturation), trailCoreMask(sdfTrail, mod));
    trail = applyTrailLayer(trail, saturate(accentColor, 1.5), cursorGlowMask(sdfCurrentCursor, .002, 0.004));
    trail = applyTrailLayer(trail, saturate(trailColor, 1.5), cursorEdgeMask(sdfCurrentCursor, .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
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

    float sdfCurrentCursor = getSdfRectangle(vu, currentCursor.xy - (currentCursor.zw * offsetFactor), currentCursor.zw * 0.5);
    float sdfTrail = getSdfParallelogram(vu, v0, v1, v2, v3);

    float progress = clamp((iTime - iTimeCursorChange) / duration, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    float mod = .005;
    vec2 dir = normalize(vu - centerCC + 1e-6);
    float splitAxis = mix(dir.x, dir.y, clamp(horizontal, 0.0, 1.0));
    float hardMix = step(0.0, splitAxis);
    float softMix = smoothstep(-0.08, 0.08, splitAxis);
    float splitMix = mix(hardMix, softMix, clamp(blendEnabled, 0.0, 1.0));
    float pulse = 0.05 * sin(iTime * 1.6) * clamp(blendEnabled, 0.0, 1.0);
    float edgeMix = clamp(splitMix + pulse * 0.45, 0.0, 1.0);

    vec4 base = mix(color0, color1, splitMix);
    vec4 edge = mix(color0, color1, edgeMix);

    vec4 trail = fragColor;
    trail = applyTrailLayer(trail, saturate(base, 1.4), trailGlowMask(sdfTrail, mod + 0.010, 0.035));
    trail = applyTrailLayer(trail, saturate(edge, 1.55), trailEdgeMask(sdfTrail, mod, 0.006));
    trail = mix(trail, saturate(base, 1.45), trailCoreMask(sdfTrail, mod));

    trail = applyTrailLayer(trail, saturate(edge, 1.55), cursorGlowMask(sdfCurrentCursor, .002, 0.004));
    trail = applyTrailLayer(trail, saturate(base, 1.5), cursorEdgeMask(sdfCurrentCursor, .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}

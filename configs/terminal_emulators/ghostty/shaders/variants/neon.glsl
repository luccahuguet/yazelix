// Neon variant with a warm synthwave stack and cyan as a supporting accent

const vec4 NEON_CYAN = vec4(0.00, 0.92, 1.00, 1.0);
const vec4 NEON_PURPLE = vec4(0.60, 0.20, 1.00, 1.0);
const vec4 NEON_RED = vec4(1.00, 0.24, 0.28, 1.0);
const vec4 NEON_YELLOW = vec4(1.00, 0.88, 0.18, 1.0);
const vec4 NEON_CORE_SHADOW = vec4(0.02, 0.08, 0.12, 1.0);
const float DURATION = 0.22;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
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

    float progress = clamp((iTime - iTimeCursorChange) / DURATION, 0.0, 1.0);
    float easedProgress = ease(progress);
    float lineLength = distance(centerCC, centerCP);

    float mod = .005;

    float pulse = 0.06 * sin(iTime * 1.2);
    vec4 neonCore = mix(NEON_CYAN, vec4(0.78, 1.00, 1.00, 1.0), 0.16 + pulse * 0.10);
    vec4 neonBase = mix(
        mix(NEON_PURPLE, NEON_RED, 0.42 + pulse * 0.10),
        mix(NEON_YELLOW, NEON_CYAN, 0.18),
        0.30
    );
    vec4 neonBorder = mix(NEON_RED, vec4(1.00, 0.50, 0.10, 1.0), 0.24 + pulse * 0.08);

    float cursorOuterPurpleGlow = cursorGlowMask(sdfCurrentCursor, .016, 0.022);
    float cursorOuterRedGlow = cursorGlowMask(sdfCurrentCursor, .011, 0.016);
    float cursorOuterYellowGlow = cursorGlowMask(sdfCurrentCursor, .007, 0.011);
    float cursorOuterCyanGlow = cursorGlowMask(sdfCurrentCursor, .004, 0.006);
    float cursorInnerGlow = cursorGlowMask(sdfCurrentCursor, .006, 0.006);
    float cursorPurpleRing = clamp(
        (1.0 - smoothstep(-0.008, -0.001, sdfCurrentCursor))
        - (1.0 - smoothstep(-0.020, -0.011, sdfCurrentCursor)),
        0.0,
        1.0
    );
    float cursorRedInnerHalo = clamp(
        (1.0 - smoothstep(-0.003, 0.003, sdfCurrentCursor))
        - (1.0 - smoothstep(-0.024, -0.014, sdfCurrentCursor)),
        0.0,
        1.0
    );
    float cursorRedBorder = cursorEdgeMask(sdfCurrentCursor, -.0032, 0.0155);
    float cursorYellowBorder = cursorEdgeMask(sdfCurrentCursor, -.0002, 0.0062);
    float cursorCore = clamp(1.0 - smoothstep(-0.026, -0.012, sdfCurrentCursor), 0.0, 1.0);

    // Build a duo trail: cyan fill and halo with a red accented rim
    vec4 trail = fragColor;
    trail = applyTrailLayer(trail, saturate(neonBase, 1.6), trailGlowMask(sdfTrail, mod + 0.012, 0.045));
    trail = applyTrailLayer(trail, saturate(neonBorder, 1.8), trailEdgeMask(sdfTrail, mod, 0.005));
    trail = mix(trail, saturate(neonBase, 1.2), trailCoreMask(sdfTrail, mod) * 0.50);

    // Make the cursor read like a neon tube: cyan inner face with a warm layered synthwave halo
    trail = mix(trail, mix(fragColor, NEON_CORE_SHADOW, 0.70), cursorCore * 0.55);
    trail = applyTrailLayer(trail, saturate(neonCore, 1.65), cursorCore * 0.92);
    trail = applyTrailLayer(trail, saturate(NEON_PURPLE, 2.0), cursorOuterPurpleGlow * 0.95);
    trail = applyTrailLayer(trail, saturate(NEON_RED, 2.05), cursorOuterRedGlow * 0.88);
    trail = applyTrailLayer(trail, saturate(NEON_YELLOW, 2.1), cursorOuterYellowGlow * 0.62);
    trail = applyTrailLayer(trail, saturate(NEON_CYAN, 1.45), cursorOuterCyanGlow * 0.18);
    trail = applyTrailLayer(trail, saturate(mix(neonCore, neonBase, 0.25), 1.7), cursorInnerGlow * 0.65);
    trail = applyTrailLayer(trail, saturate(NEON_PURPLE, 2.15), cursorPurpleRing);
    trail = applyTrailLayer(trail, saturate(neonBorder, 2.35), cursorRedInnerHalo);
    trail = applyTrailLayer(trail, saturate(NEON_RED, 2.3), cursorRedBorder);
    trail = applyTrailLayer(trail, saturate(NEON_YELLOW, 2.65), cursorYellowBorder);
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}

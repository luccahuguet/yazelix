// Inferno variant: duo horizon (blazing crimson ↑, gunmetal steel ↓)

const vec4 INFERNO_CRIMSON = vec4(1.0, 0.086, 0.0, 1.0);    // #FF1600
const vec4 INFERNO_GUNMETAL = vec4(0.165, 0.200, 0.245, 1.0); // #2A3340
const float DURATION = 0.24;

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

    vec2 dir = normalize(vu - centerCC + 1e-6);
    float angle = atan(dir.y, dir.x);
    float verticalMix = smoothstep(-0.08, 0.08, dir.y);
    float pulse = 0.05 * sin(iTime * 1.8);
    float edgeMix = clamp(verticalMix + pulse * 0.5, 0.0, 1.0);

    vec4 base = mix(INFERNO_CRIMSON, INFERNO_GUNMETAL, verticalMix);
    vec4 edge = mix(INFERNO_CRIMSON, INFERNO_GUNMETAL, edgeMix);

    vec4 trail = fragColor;
    trail = mix(saturate(base, 1.6), trail, 1. - smoothstep(0.0, sdfTrail + mod + 0.010, 0.035));
    trail = mix(saturate(edge, 1.7), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    trail = mix(trail, saturate(base, 1.65), step(sdfTrail + mod, 0.));

    trail = mix(saturate(edge, 1.7), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(INFERNO_CRIMSON, trail, 1. - smoothstep(0., sdfCurrentCursor + .001, 0.003));
    float fade = 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength);
    fragColor = mix(trail, fragColor, fade);
    float coreMask = 1. - smoothstep(-0.0015, 0.0005, sdfCurrentCursor);
    fragColor = mix(fragColor, INFERNO_CRIMSON, coreMask);
}

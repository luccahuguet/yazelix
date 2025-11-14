// Orchid variant: duo orbit (molten amber ↔ cobalt flash)

vec4 dualBlend(float segment, vec4 c0, vec4 c1) {
    float seg = mod(segment, 2.0);
    float frac = fract(seg);
    vec4 a = (seg < 1.0) ? c0 : c1;
    vec4 b = (seg < 1.0) ? c1 : c0;
    float blend = smoothstep(0.0, 1.0, frac);
    return mix(a, b, blend);
}

const vec4 ORCHID_AMBER = vec4(1.0, 0.420, 0.0, 1.0);      // #FF6B00
const vec4 ORCHID_COBALT = vec4(0.126, 0.427, 0.808, 1.0); // #206DCE
const float DURATION = 0.25;

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
    float normAngle = (angle + 3.14159265) / (6.2831853);
    float segment = normAngle * 2.0;
    float pulse = 0.05 * sin(iTime * 1.4);

    vec4 base = dualBlend(segment, ORCHID_AMBER, ORCHID_COBALT);
    vec4 edge = dualBlend(segment + 0.5 + pulse * 0.2, ORCHID_AMBER, ORCHID_COBALT);

    vec4 trail = fragColor;
    trail = mix(saturate(base, 1.5), trail, 1. - smoothstep(0.0, sdfTrail + mod + 0.010, 0.035));
    trail = mix(saturate(edge, 1.6), trail, 1. - smoothstep(0., sdfTrail + mod, 0.006));
    trail = mix(trail, saturate(base, 1.55), step(sdfTrail + mod, 0.));

    trail = mix(saturate(edge, 1.6), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    trail = mix(saturate(base, 1.55), trail, 1. - smoothstep(0., sdfCurrentCursor + .002, 0.004));
    fragColor = mix(trail, fragColor, 1. - smoothstep(0., sdfCurrentCursor, easedProgress * lineLength));
}

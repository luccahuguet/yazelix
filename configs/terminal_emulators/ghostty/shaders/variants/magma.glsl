// Magma variant: horizontal split (blazing crimson / gunmetal steel)

const vec4 MAGMA_CRIMSON = vec4(1.0, 0.086, 0.0, 1.0);    // #FF1600
const vec4 MAGMA_GUNMETAL = vec4(0.165, 0.200, 0.245, 1.0); // #2A3340
const float DURATION = 0.24;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSplitColorTrail(fragColor, fragCoord, MAGMA_CRIMSON, MAGMA_GUNMETAL, DURATION, 1.0, 1.0);
}

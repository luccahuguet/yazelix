// Reef variant: vertical split (electric cyan / intense green)

const vec4 REEF_CYAN = vec4(0.0, 0.902, 1.0, 1.0);   // #00E6FF
const vec4 REEF_GREEN = vec4(0.0, 1.0, 0.400, 1.0);   // #00FF66
const float DURATION = 0.26;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSplitColorTrail(fragColor, fragCoord, REEF_CYAN, REEF_GREEN, DURATION, 0.0, 1.0);
}

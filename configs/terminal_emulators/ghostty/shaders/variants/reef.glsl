// Reef variant: duo orbit (electric cyan ↔ venom green)

const vec4 REEF_CYAN = vec4(0.0, 0.902, 1.0, 1.0);   // #00E6FF
const vec4 REEF_VENOM = vec4(0.0, 0.752, 0.231, 1.0); // #00C03B
const float DURATION = 0.26;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderOrbitDualColorTrail(fragColor, fragCoord, REEF_CYAN, REEF_VENOM, DURATION, 1.5);
}

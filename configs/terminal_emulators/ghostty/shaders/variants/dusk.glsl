// Dusk variant: midnight blue core with coral highlights

const vec4 DUSK_BLUE = vec4(0.118, 0.118, 0.184, 1.0); // #1E1E2F
const vec4 DUSK_CORAL = vec4(0.914, 0.271, 0.376, 1.0); // #E94560
const float DURATION = 0.22;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderAxisGradientTrail(fragColor, fragCoord, DUSK_BLUE, DUSK_CORAL, DURATION, 1.3, 0.40, 0.32, 0.6);
}

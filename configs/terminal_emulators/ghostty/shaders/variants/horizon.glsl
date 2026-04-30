// Horizon variant: horizontal split (midnight blue ←, rose red →)

const vec4 HORIZON_MIDNIGHT = vec4(0.059, 0.204, 0.376, 1.0); // #0F3460
const vec4 HORIZON_ROSE = vec4(0.914, 0.271, 0.376, 1.0);     // #E94560
const float DURATION = 0.24;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderHorizontalSplitTrail(fragColor, fragCoord, HORIZON_MIDNIGHT, HORIZON_ROSE, DURATION, 1.6);
}

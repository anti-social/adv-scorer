package dev.evo.advscorer;

public class AdvScorer {
    public static void calcScores(
        float[] scores,
        float[] advWeights,
        boolean[] prosaleOnlyFlags,
        float minScore,
        float maxScore,
        float minAdvBoost,
        float maxAdvBoost,
        float slope,
        float intercept
    ) {
        assert scores.length == advWeights.length;
        assert scores.length == prosaleOnlyFlags.length;

        for (int i = 0; i < scores.length; i++) {
            float score;
            if (!prosaleOnlyFlags[i]) {
                score = scores[i];
            } else if (scores[i] <= 0f && scores[i] < minScore) {
                score = -1f;
            } else {
                score = maxScore * Math.min(
                    Math.max(
                        advWeights[i] * slope + intercept,
                        minAdvBoost
                    ),
                    maxAdvBoost
                );
            }
            scores[i] = score;
        }
    }
}

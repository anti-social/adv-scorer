package dev.evo.advscorer;

import java.nio.ByteBuffer;

public class AdvScorerJni {
    static {
        System.loadLibrary("advscorer");
    }

    public static native void calcScores(
        int size,
        ByteBuffer scores,
        ByteBuffer advWeights,
        ByteBuffer prosaleOnlyFlags,
        float minScore,
        float maxScore,
        float minAdvBoost,
        float maxAdvBoost,
        float slope,
        float intercept
    );
}

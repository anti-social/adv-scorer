package dev.evo.elasticsearch;

import java.nio.ByteOrder;
import java.nio.ByteBuffer;

import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.Fork;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.State;

@Fork(
        jvmArgsAppend = {
                "-XX:+UseSuperWord",
                "-XX:+UnlockDiagnosticVMOptions"
                //"-XX:CompileCommand=print,*AdvScorerBenchmarks.*"
        }
)
public class AdvScorerBenchmarks {
    @State(Scope.Benchmark)
    public static class Data {
        private static final int BATCH_SIZE = 1024;
        private final float[] scores;
        private final float[] advWeights;
        private final boolean[] prosaleOnlyFlags;
        private final float minScore;
        private final float maxScore;
        private final float slope;
        private final float intercept;
        private final float minAdvBoost;
        private final float maxAdvBoost;

        private UnsafeBuffer scoresBuf = new UnsafeBuffer(
                ByteBuffer.allocateDirect(BATCH_SIZE * 4)
                        .order(ByteOrder.LITTLE_ENDIAN)
        );
        private UnsafeBuffer advWeightsBuf = new UnsafeBuffer(
                ByteBuffer.allocateDirect(BATCH_SIZE * 4)
                        .order(ByteOrder.LITTLE_ENDIAN)
        );
        private ByteBuffer prosaleOnlyFlagsBuf = ByteBuffer.allocateDirect(BATCH_SIZE / 8)
                .order(ByteOrder.nativeOrder());

        public Data() {
            scores = new float[BATCH_SIZE];
            for (int i = 0; i < BATCH_SIZE; i++) {
                scores[i] = (float) i;
            }

            advWeights = new float[BATCH_SIZE];
            for (int i = 0; i < BATCH_SIZE; i++) {
                advWeights[i] = 1.0f / (float) i;
            }

            prosaleOnlyFlags = new boolean[BATCH_SIZE];
            for (int i = 0; i < BATCH_SIZE; i++) {
                prosaleOnlyFlags[i] = i % 2 == 0;
            }

            minScore = 1f;
            maxScore = 100f;
            slope = 0.2f;
            intercept = 0.5f;
            minAdvBoost = 10f;
            maxAdvBoost = 200f;
        }
    }

    @Benchmark
    public void calcScoresJava(Data data) {
        for (int i = 0; i < Data.BATCH_SIZE; i++) {
            float score;
            if (!data.prosaleOnlyFlags[i]) {
                score = data.scores[i];
            } else if (data.scores[i] <= 0f && data.scores[i] < data.minScore) {
                score = -1f;
            } else {
                score = data.maxScore * Math.min(
                        Math.max(
                                data.advWeights[i] * data.slope + data.intercept,
                                data.minAdvBoost
                        ),
                        data.maxAdvBoost
                );
            }
            data.scores[i] = score;
        }
    }

//    @Benchmark
//    public void copyToByteBuffer(Data data) {
//        for (int i = 0; i < Data.BATCH_SIZE; i++) {
//            data.scoresBuf.writeFloat(i * 4, data.scores[i]);
//            data.advWeightsBuf.writeFloat(i * 4, data.advWeights[i]);
//        }
//    }
}

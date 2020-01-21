package dev.evo.advscorer;

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
        private static final int BATCH_SIZE = 1024 * 64;
        private final float[] scores;
        private final float[] advWeights;
        private final boolean[] prosaleOnlyFlags;
        private final float minScore;
        private final float maxScore;
        private final float slope;
        private final float intercept;
        private final float minAdvBoost;
        private final float maxAdvBoost;

        private ByteBuffer scoresBuf = AlignedBuffer.create(BATCH_SIZE * 4, 32);
        private ByteBuffer advWeightsBuf = AlignedBuffer.create(BATCH_SIZE * 4, 32);
        private ByteBuffer prosaleOnlyFlagsBuf = AlignedBuffer.create(BATCH_SIZE * 4, 32);

        public Data() {
            scores = new float[BATCH_SIZE];
            for (int i = 0; i < BATCH_SIZE; i++) {
                scores[i] = (float) i;
                scoresBuf.putFloat(i * 4, scores[i]);
            }

            advWeights = new float[BATCH_SIZE];
            for (int i = 0; i < BATCH_SIZE; i++) {
                advWeights[i] = 1.0f / (float) i;
                advWeightsBuf.putFloat(i * 4, advWeights[i]);
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
        AdvScorer.calcScores(
            data.scores, data.advWeights, data.prosaleOnlyFlags,
            data.minScore, data.maxScore,
            data.minAdvBoost, data.maxAdvBoost,
            data.slope, data.intercept
        );
    }

//    @Benchmark
//    public void calcScoresJni(Data data) {
//        AdvScorerJni.calcScores(
//            Data.BATCH_SIZE,
//            data.scoresBuf, data.advWeightsBuf, data.prosaleOnlyFlagsBuf,
//            data.minScore, data.maxScore,
//            data.minAdvBoost, data.maxAdvBoost,
//            data.slope, data.intercept
//        );
//    }

//    @Benchmark
//    public void copyToByteBuffer(Data data) {
//        for (int i = 0; i < Data.BATCH_SIZE; i++) {
//            data.scoresBuf.writeFloat(i * 4, data.scores[i]);
//            data.advWeightsBuf.writeFloat(i * 4, data.advWeights[i]);
//        }
//    }
}

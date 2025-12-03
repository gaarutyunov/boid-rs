#include "wrapper.h"
#include <memory>
#include <vector>

// MediaPipe includes
#include "mediapipe/framework/calculator_framework.h"
#include "mediapipe/framework/formats/image_frame.h"
#include "mediapipe/framework/formats/image_frame_opencv.h"
#include "mediapipe/framework/port/opencv_core_inc.h"
#include "mediapipe/framework/port/opencv_imgproc_inc.h"
#include "mediapipe/framework/port/parse_text_proto.h"
#include "mediapipe/framework/port/status.h"
#include "mediapipe/calculators/core/flow_limiter_calculator.pb.h"
#include "mediapipe/calculators/util/landmarks_to_render_data_calculator.pb.h"
#include "mediapipe/calculators/util/rect_to_render_data_calculator.pb.h"
#include "mediapipe/framework/formats/landmark.pb.h"

// MediaPipe hand tracking graph configuration
constexpr char kInputStream[] = "input_video";
constexpr char kOutputStream[] = "hand_landmarks";
constexpr char kWindowName[] = "MediaPipe";

constexpr char kGraphConfigText[] = R"pb(
  input_stream: "input_video"
  output_stream: "hand_landmarks"
  output_stream: "handedness"

  node {
    calculator: "HandLandmarkTrackingCpu"
    input_stream: "IMAGE:input_video"
    output_stream: "LANDMARKS:hand_landmarks"
    output_stream: "HANDEDNESS:handedness"
    node_options: {
      [type.googleapis.com/mediapipe.HandLandmarkTrackingCpuOptions] {
        num_hands: 2
        min_detection_confidence: 0.5
        min_tracking_confidence: 0.5
      }
    }
  }
)pb";

struct MediaPipeHandDetector {
    std::unique_ptr<mediapipe::CalculatorGraph> graph;
    bool initialized;

    MediaPipeHandDetector() : initialized(false) {}
};

extern "C" {

MediaPipeHandDetector* mediapipe_hand_detector_create() {
    auto detector = std::make_unique<MediaPipeHandDetector>();

    // Parse the graph configuration
    mediapipe::CalculatorGraphConfig config;
    if (!mediapipe::ParseTextProto<mediapipe::CalculatorGraphConfig>(
            kGraphConfigText, &config)) {
        return nullptr;
    }

    // Create and initialize the graph
    detector->graph = std::make_unique<mediapipe::CalculatorGraph>();
    auto status = detector->graph->Initialize(config);
    if (!status.ok()) {
        return nullptr;
    }

    status = detector->graph->StartRun({});
    if (!status.ok()) {
        return nullptr;
    }

    detector->initialized = true;
    return detector.release();
}

void mediapipe_hand_detector_destroy(MediaPipeHandDetector* detector) {
    if (detector) {
        if (detector->initialized && detector->graph) {
            detector->graph->CloseAllInputStreams();
            detector->graph->WaitUntilDone();
        }
        delete detector;
    }
}

int mediapipe_hand_detector_process(
    MediaPipeHandDetector* detector,
    const unsigned char* image_data,
    int width,
    int height,
    MediaPipeHand* hands,
    int max_hands
) {
    if (!detector || !detector->initialized || !image_data || !hands) {
        return 0;
    }

    // Convert image data to OpenCV Mat (BGR format)
    cv::Mat input_frame(height, width, CV_8UC3, const_cast<unsigned char*>(image_data));

    // Convert to ImageFrame
    auto input_frame_mediapipe = absl::make_unique<mediapipe::ImageFrame>(
        mediapipe::ImageFormat::SRGB, width, height,
        mediapipe::ImageFrame::kDefaultAlignmentBoundary);
    cv::Mat input_frame_mat = mediapipe::formats::MatView(input_frame_mediapipe.get());
    cv::cvtColor(input_frame, input_frame_mat, cv::COLOR_BGR2RGB);

    // Send packet to graph
    size_t frame_timestamp_us =
        (double)cv::getTickCount() / (double)cv::getTickFrequency() * 1e6;
    auto status = detector->graph->AddPacketToInputStream(
        kInputStream,
        mediapipe::Adopt(input_frame_mediapipe.release())
            .At(mediapipe::Timestamp(frame_timestamp_us)));

    if (!status.ok()) {
        return 0;
    }

    // Get output packet
    mediapipe::Packet packet;
    if (!detector->graph->GetOutputStreamPacket(kOutputStream, &packet).ok()) {
        return 0;
    }

    // Extract hand landmarks
    const auto& output_landmarks = packet.Get<std::vector<mediapipe::NormalizedLandmarkList>>();

    int num_hands = std::min(static_cast<int>(output_landmarks.size()), max_hands);

    for (int h = 0; h < num_hands; ++h) {
        const auto& landmarks = output_landmarks[h];
        for (int i = 0; i < 21 && i < landmarks.landmark_size(); ++i) {
            const auto& landmark = landmarks.landmark(i);
            hands[h].landmarks[i].x = landmark.x();
            hands[h].landmarks[i].y = landmark.y();
            hands[h].landmarks[i].z = landmark.z();
            hands[h].landmarks[i].visibility = landmark.visibility();
            hands[h].landmarks[i].presence = landmark.presence();
        }
        hands[h].handedness = h; // Simplified handedness
    }

    return num_hands;
}

} // extern "C"

export default function ScoreGauge({ score = 0, size = "md" }) {
  const getColor = (s) => {
    if (s >= 80) return "#00FF41";
    if (s >= 60) return "#00D4FF";
    if (s >= 40) return "#FFB800";
    return "#FF0055";
  };

  const dimensions =
    size === "sm" ? "w-8 h-8 text-[10px]" : "w-12 h-12 text-xs";

  return (
    <div
      className={`${dimensions} rounded-full border-2 flex items-center justify-center font-bold shrink-0`}
      style={{ borderColor: getColor(score), color: getColor(score) }}
    >
      {score}
    </div>
  );
}

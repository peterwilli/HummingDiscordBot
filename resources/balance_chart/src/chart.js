import {
  Chart,
  TimeScale,
  LineController,
  LinearScale,
  Legend,
  PointElement,
  LineElement,
  Decimation,
} from "chart.js";
Chart.register(
  TimeScale,
  LineController,
  LinearScale,
  Legend,
  PointElement,
  LineElement,
  Decimation,
);
import "chartjs-adapter-moment";

const colors = {
  purple: {
    default: "rgba(149, 76, 233, 1)",
    half: "rgba(149, 76, 233, 0.5)",
    quarter: "rgba(149, 76, 233, 0.25)",
    zero: "rgba(149, 76, 233, 0)",
  },
  indigo: {
    default: "rgba(80, 102, 120, 1)",
    quarter: "rgba(80, 102, 120, 0.25)",
  },
};
const botDataStr = decodeURIComponent(window.location.hash.substring(1));
const botData = JSON.parse(botDataStr);
const ctx = document.querySelector(".chart canvas").getContext("2d");
const gradient = ctx.createLinearGradient(0, 25, 0, 300);
gradient.addColorStop(0, colors.purple.half);
gradient.addColorStop(0.35, colors.purple.quarter);
gradient.addColorStop(1, colors.purple.zero);

let draw = LineController.prototype.draw;
LineController.prototype.draw = function () {
  let chart = this.chart;
  let ctx = chart.ctx;
  let _stroke = ctx.stroke;
  if ("shadowColor" in chart.options) {
    ctx.stroke = function () {
      ctx.save();
      ctx.shadowColor = chart.options.shadowColor;
      ctx.shadowBlur = 20;
      ctx.shadowOffsetX = 0;
      ctx.shadowOffsetY = 0;
      _stroke.apply(this, arguments);
      ctx.restore();
    };
  }
  draw.apply(this, arguments);
  ctx.stroke = _stroke;
};
Chart.defaults.color = "#ddd";

const options = {
  type: "line",
  data: {
    datasets: [
      {
        label: botData.baseAsset,
        backgroundColor: gradient,
        pointRadius: 0,
        borderColor: colors.purple.default,
        data: botData.chartData.map((entry) => {
          return {
            x: entry.timestamp,
            y: parseFloat(entry.profit),
          };
        }),
        lineTension: 0.1,
        borderWidth: 2,
      },
    ],
  },
  options: {
    animation: false,
    parsing: false,
    shadowColor: "#e15bff",
    responsive: true,
    maintainAspectRatio: false,
    layout: {
      padding: 10,
    },
    plugins: {
      decimation: {
        enabled: true,
        algorithm: 'lttb',
        samples: 1
      }
    },
    scales: {
      x: {
        type: "time",
        ticks: {
          source: 'auto',
          // Disabled rotation for performance
          maxRotation: 0,
          autoSkip: true,
        },
        title: {
          display: true,
          text: "Time",
        },
      },
      y: {
        title: {
          display: true,
          text: "Profit (Price)",
        },
      },
    },
  },
};

new Chart(ctx, options);

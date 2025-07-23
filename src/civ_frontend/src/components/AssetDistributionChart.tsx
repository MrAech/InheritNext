import { PieChart, Pie, Cell, ResponsiveContainer, Legend } from "recharts";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";

interface Asset {
  id: string;
  name: string;
  value: number;
}

interface Heir {
  id: string;
  name: string;
}

interface AssetDistribution {
  id: string;
  assetId: string;
  heirId: string;
  percentage: number;
}

interface AssetDistributionChartProps {
  asset: Asset;
  heirs: Heir[];
  distributions: AssetDistribution[];
}

const COLORS = [
  "hsl(var(--chart-1))",
  "hsl(var(--chart-2))",
  "hsl(var(--chart-3))",
  "hsl(var(--chart-4))",
  "hsl(var(--chart-5))",
];

const AssetDistributionChart = ({ asset, heirs, distributions }: AssetDistributionChartProps) => {
  const assetDistributions = distributions.filter(d => d.assetId === asset.id);
  const totalDistributed = assetDistributions.reduce((sum, d) => sum + d.percentage, 0);
  const remaining = 100 - totalDistributed;

  const chartData = [
    ...assetDistributions.map((distribution, index) => {
      const heir = heirs.find(h => h.id === distribution.heirId);
      return {
        name: heir?.name || "Unknown Heir",
        value: distribution.percentage,
        fill: COLORS[index % COLORS.length],
      };
    }),
    ...(remaining > 0 ? [{
      name: "Unallocated",
      value: remaining,
      fill: "hsl(var(--muted))",
    }] : [])
  ];

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const chartConfig = {
    value: {
      label: "Percentage",
    },
  };

  return (
    <div className="space-y-4">
      <div className="text-center">
        <h4 className="text-sm font-medium text-muted-foreground mb-2">
          Distribution for {asset.name}
        </h4>
        <p className="text-xs text-muted-foreground">
          Total Value: {formatCurrency(asset.value)}
        </p>
      </div>

      <ChartContainer config={chartConfig} className="h-[300px]">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={chartData}
              cx="50%"
              cy="50%"
              innerRadius={60}
              outerRadius={120}
              paddingAngle={2}
              dataKey="value"
            >
              {chartData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.fill} />
              ))}
            </Pie>
            <ChartTooltip
              content={({ active, payload }) => {
                if (active && payload && payload.length) {
                  const data = payload[0];
                  const value = data.value as number;
                  const name = data.payload?.name;
                  const inheritanceValue = (Number(asset.value) * value) / 100;

                  return (
                    <div className="rounded-lg border bg-background p-2 shadow-sm">
                      <div className="grid gap-2">
                        <div className="flex items-center justify-between gap-2">
                          <div className="flex items-center gap-1.5">
                            <div
                              className="h-2.5 w-2.5 shrink-0 rounded-[2px]"
                              style={{ backgroundColor: data.payload?.fill }}
                            />
                            <span className="text-muted-foreground">{name}</span>
                          </div>
                          <span className="font-mono font-medium tabular-nums text-foreground">
                            {value}%
                          </span>
                        </div>
                        {name !== "Unallocated" && (
                          <div className="text-xs text-muted-foreground">
                            Value: {formatCurrency(inheritanceValue)}
                          </div>
                        )}
                      </div>
                    </div>
                  );
                }
                return null;
              }}
            />
            <Legend
              verticalAlign="bottom"
              height={36}
              formatter={(value, entry) => (
                <span className="text-sm" style={{ color: entry.color }}>
                  {value} ({entry.payload?.value}%)
                </span>
              )}
            />
          </PieChart>
        </ResponsiveContainer>
      </ChartContainer>
    </div>
  );
};

export { AssetDistributionChart };

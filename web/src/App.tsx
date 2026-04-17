import { useEffect, useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Activity, FolderTree, GitCommitHorizontal, Users } from "lucide-react";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./components/ui/card";
import { Input } from "./components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./components/ui/table";

type AuthorStat = {
  name: string;
  email: string;
  commit_count: number;
  effective_commit_count: number;
  additions: number;
  deletions: number;
  net_lines: number;
};

type Summary = {
  total_authors: number;
  total_commits: number;
  effective_commits: number;
  total_additions: number;
  total_deletions: number;
  net_lines: number;
};

type TimeseriesPoint = {
  date: string;
  commits: number;
  additions: number;
  deletions: number;
  net_lines: number;
};

type PathStat = {
  path: string;
  commits: number;
  additions: number;
  deletions: number;
  net_lines: number;
};

type Dashboard = {
  report: {
    meta: {
      repo: string;
      branch: string;
      since: string | null;
      until: string | null;
    };
    summary: Summary;
    authors: AuthorStat[];
  };
  timeseries: TimeseriesPoint[];
  paths: PathStat[];
};

type Filters = {
  since: string;
  until: string;
  noMerge: boolean;
  excludeDir: string;
  excludeExt: string;
};

const PIE_COLORS = ["#1e5b52", "#e8a54b", "#2563eb", "#ca8a04", "#7c3aed"];

export default function App() {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>({
    since: "",
    until: "",
    noMerge: false,
    excludeDir: "",
    excludeExt: "",
  });

  async function loadDashboard(nextFilters: Filters) {
    setLoading(true);
    setError(null);

    const params = new URLSearchParams();
    if (nextFilters.since) params.set("since", nextFilters.since);
    if (nextFilters.until) params.set("until", nextFilters.until);
    if (nextFilters.noMerge) params.set("no_merge", "true");
    if (nextFilters.excludeDir) params.append("exclude_dir", nextFilters.excludeDir);
    if (nextFilters.excludeExt) params.append("exclude_ext", nextFilters.excludeExt);

    const response = await fetch(`/api/dashboard?${params.toString()}`);
    if (!response.ok) {
      throw new Error(`Failed to load dashboard: ${response.status}`);
    }
    const json = (await response.json()) as Dashboard;
    setDashboard(json);
    setLoading(false);
  }

  useEffect(() => {
    loadDashboard(filters).catch((err: Error) => {
      setError(err.message);
      setLoading(false);
    });
  }, []);

  const topAuthors = useMemo(() => {
    return (dashboard?.report.authors ?? []).slice(0, 5).map((author) => ({
      name: author.name,
      commits: author.effective_commit_count,
    }));
  }, [dashboard]);

  const contributionShare = useMemo(() => {
    return (dashboard?.report.authors ?? []).slice(0, 5).map((author) => ({
      name: author.name,
      value: author.additions,
    }));
  }, [dashboard]);

  const summary = dashboard?.report.summary;
  const meta = dashboard?.report.meta;

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(232,165,75,0.18),_transparent_24%),linear-gradient(180deg,_#faf8f1_0%,_#f4efe4_100%)] text-foreground">
      <div className="mx-auto flex max-w-7xl flex-col gap-6 px-4 py-8 md:px-8">
        <section className="grid gap-6 lg:grid-cols-[1.6fr_1fr]">
          <Card className="overflow-hidden bg-[linear-gradient(135deg,_rgba(30,91,82,0.95),_rgba(23,32,51,0.95))] text-white">
            <CardHeader className="gap-3">
              <Badge className="w-fit bg-white/10 text-white">git-report web</Badge>
              <CardTitle className="text-3xl md:text-4xl">
                本地 Git 统计面板
              </CardTitle>
              <CardDescription className="max-w-2xl text-sm text-white/80">
                快速浏览提交趋势、作者贡献和路径热点。所有分析都在本地完成。
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4 text-sm md:grid-cols-3">
              <MetaItem label="Repository" value={meta?.repo ?? "Loading…"} />
              <MetaItem label="Branch" value={meta?.branch ?? "Loading…"} />
              <MetaItem
                label="Range"
                value={`${meta?.since ?? "default"} → ${meta?.until ?? "now"}`}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>筛选器</CardTitle>
              <CardDescription>修改参数后刷新 dashboard。</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-3">
              <Input
                placeholder='Since, e.g. "30 days ago"'
                value={filters.since}
                onChange={(e) => setFilters((prev) => ({ ...prev, since: e.target.value }))}
              />
              <Input
                placeholder="Until, e.g. 2026-04-30"
                value={filters.until}
                onChange={(e) => setFilters((prev) => ({ ...prev, until: e.target.value }))}
              />
              <Input
                placeholder="Exclude directory fragment"
                value={filters.excludeDir}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, excludeDir: e.target.value }))
                }
              />
              <Input
                placeholder="Exclude extension, e.g. .spec.ts"
                value={filters.excludeExt}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, excludeExt: e.target.value }))
                }
              />
              <label className="flex items-center gap-2 rounded-lg border border-border bg-muted/50 px-3 py-2 text-sm">
                <input
                  type="checkbox"
                  checked={filters.noMerge}
                  onChange={(e) =>
                    setFilters((prev) => ({ ...prev, noMerge: e.target.checked }))
                  }
                />
                Exclude merge commits
              </label>
              <Button
                onClick={() =>
                  loadDashboard(filters).catch((err: Error) => {
                    setError(err.message);
                    setLoading(false);
                  })
                }
              >
                Refresh Dashboard
              </Button>
            </CardContent>
          </Card>
        </section>

        {error ? (
          <Card className="border-red-300 bg-red-50">
            <CardContent className="p-6 text-red-700">{error}</CardContent>
          </Card>
        ) : null}

        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <KpiCard
            title="Authors"
            icon={<Users className="h-4 w-4" />}
            value={summary?.total_authors ?? 0}
            accent="bg-emerald-100 text-emerald-900"
          />
          <KpiCard
            title="Effective Commits"
            icon={<GitCommitHorizontal className="h-4 w-4" />}
            value={summary?.effective_commits ?? 0}
            accent="bg-amber-100 text-amber-900"
          />
          <KpiCard
            title="Net Lines"
            icon={<Activity className="h-4 w-4" />}
            value={summary?.net_lines ?? 0}
            accent="bg-blue-100 text-blue-900"
          />
          <KpiCard
            title="Tracked Paths"
            icon={<FolderTree className="h-4 w-4" />}
            value={dashboard?.paths.length ?? 0}
            accent="bg-violet-100 text-violet-900"
          />
        </section>

        <section className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
          <Card>
            <CardHeader>
              <CardTitle>Commit & Line Trend</CardTitle>
              <CardDescription>按天查看提交量和增删行变化。</CardDescription>
            </CardHeader>
            <CardContent className="h-[320px]">
              {loading ? (
                <PanelPlaceholder />
              ) : (
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={dashboard?.timeseries ?? []}>
                    <defs>
                      <linearGradient id="additions" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor="#1e5b52" stopOpacity={0.8} />
                        <stop offset="95%" stopColor="#1e5b52" stopOpacity={0.05} />
                      </linearGradient>
                      <linearGradient id="deletions" x1="0" x2="0" y1="0" y2="1">
                        <stop offset="5%" stopColor="#e8a54b" stopOpacity={0.7} />
                        <stop offset="95%" stopColor="#e8a54b" stopOpacity={0.05} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid stroke="#d5d0bf" strokeDasharray="3 3" />
                    <XAxis dataKey="date" stroke="#64748b" />
                    <YAxis stroke="#64748b" />
                    <Tooltip />
                    <Area type="monotone" dataKey="additions" stroke="#1e5b52" fill="url(#additions)" />
                    <Area type="monotone" dataKey="deletions" stroke="#e8a54b" fill="url(#deletions)" />
                  </AreaChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Author Mix</CardTitle>
              <CardDescription>按新增行数查看前五位作者占比。</CardDescription>
            </CardHeader>
            <CardContent className="h-[320px]">
              {loading ? (
                <PanelPlaceholder />
              ) : (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={contributionShare}
                      dataKey="value"
                      nameKey="name"
                      innerRadius={60}
                      outerRadius={100}
                      paddingAngle={4}
                    >
                      {contributionShare.map((entry, index) => (
                        <Cell key={entry.name} fill={PIE_COLORS[index % PIE_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>
        </section>

        <section className="grid gap-6 xl:grid-cols-[1fr_1.1fr]">
          <Card>
            <CardHeader>
              <CardTitle>Top Contributors</CardTitle>
              <CardDescription>按照有效提交数排序。</CardDescription>
            </CardHeader>
            <CardContent className="h-[320px]">
              {loading ? (
                <PanelPlaceholder />
              ) : (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={topAuthors}>
                    <CartesianGrid stroke="#d5d0bf" strokeDasharray="3 3" />
                    <XAxis dataKey="name" stroke="#64748b" />
                    <YAxis stroke="#64748b" />
                    <Tooltip />
                    <Bar dataKey="commits" radius={[8, 8, 0, 0]} fill="#1e5b52" />
                  </BarChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Author Details</CardTitle>
              <CardDescription>完整作者统计明细。</CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Author</TableHead>
                    <TableHead>Effective</TableHead>
                    <TableHead>Additions</TableHead>
                    <TableHead>Deletions</TableHead>
                    <TableHead>Net</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(dashboard?.report.authors ?? []).map((author) => (
                    <TableRow key={author.email}>
                      <TableCell>
                        <div className="font-medium">{author.name}</div>
                        <div className="text-xs text-slate-500">{author.email}</div>
                      </TableCell>
                      <TableCell>{author.effective_commit_count}</TableCell>
                      <TableCell>{author.additions}</TableCell>
                      <TableCell>{author.deletions}</TableCell>
                      <TableCell>{author.net_lines}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </section>

        <Card>
          <CardHeader>
            <CardTitle>Hot Paths</CardTitle>
            <CardDescription>最多展示前 20 个变更路径。</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Path</TableHead>
                  <TableHead>Commits</TableHead>
                  <TableHead>Additions</TableHead>
                  <TableHead>Deletions</TableHead>
                  <TableHead>Net</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {(dashboard?.paths ?? []).map((path) => (
                  <TableRow key={path.path}>
                    <TableCell className="font-mono text-xs">{path.path}</TableCell>
                    <TableCell>{path.commits}</TableCell>
                    <TableCell>{path.additions}</TableCell>
                    <TableCell>{path.deletions}</TableCell>
                    <TableCell>{path.net_lines}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </main>
  );
}

function KpiCard({
  title,
  value,
  icon,
  accent,
}: {
  title: string;
  value: number;
  icon: React.ReactNode;
  accent: string;
}) {
  return (
    <Card>
      <CardContent className="flex items-center justify-between p-6">
        <div>
          <div className="text-sm text-slate-500">{title}</div>
          <div className="mt-2 text-3xl font-semibold">{value}</div>
        </div>
        <div className={`rounded-xl p-3 ${accent}`}>{icon}</div>
      </CardContent>
    </Card>
  );
}

function MetaItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-white/10 bg-white/5 p-4">
      <div className="text-xs uppercase tracking-[0.2em] text-white/60">{label}</div>
      <div className="mt-2 break-all text-sm">{value}</div>
    </div>
  );
}

function PanelPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center rounded-xl border border-dashed border-border bg-muted/40 text-sm text-slate-500">
      Loading dashboard…
    </div>
  );
}

import sys
import json
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter

# required by HotCRP
plt.rcParams['pdf.fonttype'] = 42
plt.rcParams['ps.fonttype'] = 42

def load_stats_files():
    stats = []
    for filename in sys.argv[1:]:
        with open(filename) as f:
            try:
                stat = json.load(f)
                stats.append(stat)
                print(f'Loaded {filename}')
            except json.JSONDecodeError:
                print(f'Error parsing {filename}')
    return stats


def group_stats_by_config(stats):
    groups = {'full': [], 'argmax_ucb': [], 'byte_mutation_only': []}

    for stat in stats:
        argmax_ucb = stat.get('argmax_ucb', False)
        byte_mutation_only = stat.get('byte_mutation_only', False)
        if argmax_ucb and byte_mutation_only:
            print(f'Warning: Found unusual configuration with both flags enabled')
            continue
        elif argmax_ucb:
            groups['argmax_ucb'].append(stat)
        elif byte_mutation_only:
            groups['byte_mutation_only'].append(stat)
        else:
            groups['full'].append(stat)

    for name, group in groups.items():
        print(f'{name}: {len(group)} sessions')

    return groups


def interpolate_data(stats_group, metric, max_time=24*60*60, num_points=500):
    timepoints = np.linspace(0, max_time, num_points)
    interpolated = []

    for stat in stats_group:
        iterations = stat.get('iterations', [])
        times = [it.get('seconds_used', 0) for it in iterations]
        values = [it.get(metric, 0) for it in iterations]

        interp_values = np.interp(
            timepoints,
            times,
            values,
            left=0,
        )
        interpolated.append(interp_values)

    return (
        timepoints / 3600,
        np.median(interpolated, axis=0),
    )


def plot_metric(
    stats_groups,
    metric,
    output_file,
    break_y_axis=None,
    y_top=None,
    legend_loc='best',
):
    # https://tsitsul.in/blog/coloropt/
    colors = {
        'full': '#4053d3',
        'argmax_ucb': '#ddb310',
        'byte_mutation_only': '#b51d14',
    }
    labels = {
        'full': 'Full Setup',
        'argmax_ucb': 'Argmax-Based UCB',
        'byte_mutation_only': 'Byte Mutation Only',
    }
    metric_title = {
        'incons_count': 'Inconsistent Pairs (Median)',
    }

    timepoints = np.array([0, 24])

    if break_y_axis:
        fig, (ax_top, ax_bottom) = plt.subplots(
            2,
            1,
            figsize=(6, 4),
            sharex=True,
            gridspec_kw={'height_ratios': [6, 1], 'hspace': 0.12},
        )
        axes = [ax_top, ax_bottom]
    else:
        fig, ax = plt.subplots(figsize=(6, 4))
        axes = [ax]

    # blend overlapping lines
    for t in range(10):
        for config_name in reversed(colors):
            stats_group = stats_groups.get(config_name)
            if not stats_group:
                continue

            timepoints, median_values = interpolate_data(stats_group, metric)

            if len(timepoints) == 0:
                continue

            for i, ax in enumerate(axes):
                y = median_values
                if break_y_axis and i == 1:
                    y = np.where(y <= break_y_axis, y, np.nan)
                ax.plot(
                    timepoints,
                    y,
                    alpha=0.8**t,
                    color=colors[config_name],
                    label=labels[config_name] if t == 0 else None,
                )

    # Configure each axis
    for ax in axes:
        ax.grid(True, linestyle='--', alpha=0.7)
        ax.yaxis.set_major_formatter(
            FuncFormatter(
                lambda x, _: f'{round(x/1000)}k' if x >= 10000 else f'{round(x)}'
            )
        )

    if timepoints[-1] == 24:
        axes[0].set_xticks(np.arange(0, 25, 4))
    handles, labels = axes[0].get_legend_handles_labels()
    axes[0].legend(handles[::-1], labels[::-1], loc=legend_loc)
    axes[-1].set_xlabel('Time (hours)')

    if break_y_axis and ax_top and ax_bottom:
        ax_top.set_ylim(bottom=break_y_axis, top=y_top)
        ax_bottom.set_ylim(top=break_y_axis)

        ax_top.tick_params(bottom=False)
        ax_bottom.set_yticks([0, break_y_axis])

        ax_top.spines['bottom'].set_visible(False)
        ax_bottom.spines['top'].set_visible(False)

        # Add break markers
        d = 0.015
        kwargs = dict(transform=ax_top.transAxes, color='k', clip_on=False)
        ax_top.plot((-d, +d), (-d, +d), **kwargs)
        ax_top.plot((1 - d, 1 + d), (-d, +d), **kwargs)
        kwargs.update(transform=ax_bottom.transAxes)
        ax_bottom.plot((-d, +d), (1 - d, 1 + d), **kwargs)
        ax_bottom.plot((1 - d, 1 + d,), (1 - d, 1 + d,), **kwargs)

        fig.subplots_adjust(left=0.15)
        fig.text(0.04, 0.5, metric_title[metric], va='center', rotation='vertical')

    else:
        axes[0].set_ylabel(metric_title[metric])
        plt.tight_layout()

    plt.savefig(output_file, bbox_inches='tight', pad_inches=0)
    print(f'Plot for {metric} saved to {output_file}')
    plt.close(fig)


def calc_incons(stats_groups, total_pairs):
    total_consistent_sets = []

    for config_name, stats_group in stats_groups.items():
        if not stats_group:
            continue

        consistent_sets = []
        incons = []

        for stats in stats_group:
            consistent_set = set(map(
                lambda pair: (pair[0], pair[1]),
                stats['consistent_pairs']
            ))
            incons.append(total_pairs - len(consistent_set))
            consistent_sets.append(consistent_set)
            total_consistent_sets.append(consistent_set)

        overall_incons = total_pairs - len(set.intersection(*consistent_sets))
        median_incons = np.median(incons)
        avg_incons = np.mean(incons)

        print(f'{config_name}: {overall_incons = } {median_incons = :.1f} {avg_incons = :.1f}')

    print(f'{len(set.intersection(*total_consistent_sets)) = }')
    print(set.intersection(*total_consistent_sets))


stats = load_stats_files()
if not stats:
    print('No valid stats files provided.')
    exit(1)
stats_groups = group_stats_by_config(stats)
total_pairs = stats[0]['iterations'][-1]['incons_count'] + len(stats[0]['consistent_pairs'])
plot_metric(
    stats_groups,
    'incons_count',
    'inconsistent-pair-cdf.pdf',
    break_y_axis=1000,
    y_top=1210,
)
calc_incons(stats_groups, total_pairs)

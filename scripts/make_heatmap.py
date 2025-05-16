from common import *
import pandas as pd
import plotly.express as px
import plotly.io as pio
pio.kaleido.scope.mathjax = None

SCHEDULERS = [
    'centralized',
    'fsync',
    'ssync',
    'async-lc-atomic',
    'async-cm-atomic',
    'async-move-atomic',
    'async',
]

RANGES = {
    ('full', False): range(2,3),
    ('full', True): range(2,4),
    ('external', False): range(3,5),
    ('external', True): range(3,8),
}

MODELS = [ 'full', 'external' ]
CLASSL = [ True, False ]

def model_to_str(model, classL, ncols):
    return f"{model} {ncols}{'L' if classL else ''}"

def make_z_matrix(df):
    """ returns (z matrix, vaxis labels, haxis labels)
    """
    df['ratio'] = df['pass'] / df['total']
    data = df[['model', 'classL', 'ncols', 'sched', 'ratio']].apply(lambda row: dict(row) , axis=1).tolist()
    
    data_dict = {
        color_model:{
            classL:{
                ncols:{
                    sched: '?'
                    for sched in SCHEDULERS
                } for ncols in RANGES[(color_model, classL)]
            } for classL in CLASSL
        } for color_model in MODELS
    }
    for entry in data:
        color_model = entry['model']
        classL = entry['classL']
        ncols = entry['ncols']
        sched = entry['sched']
        ratio = entry['ratio']
        if color_model in data_dict:
            if classL in data_dict[color_model]:
                if ncols in data_dict[color_model][classL]:
                    if sched in data_dict[color_model][classL][ncols]:
                        data_dict[color_model][classL][ncols][sched] = ratio

    vaxis = [ (color_model, classL, ncols) for color_model in MODELS for classL in CLASSL for ncols in RANGES[(color_model, classL)] ]
    haxis = SCHEDULERS
    mat_z = [
        [ data_dict[color_model][classL][ncols][sched] for sched in haxis ]
        for (color_model, classL, ncols) in vaxis
    ]
    return (mat_z, vaxis, haxis)

if __name__ == '__main__':
    df = pd.read_csv(OUTCOMES_FILE)
    
    (z_matrix, v_models, h_sched) = make_z_matrix(df)
    v_labels = [ model_to_str(model, classL, ncols) for (model, classL, ncols) in v_models ]
    print(z_matrix)
    print("-" * 20)
    print(v_models)
    print("-" * 20)
    print(v_models)
    

    fig = px.imshow(z_matrix, x=h_sched, y=v_labels,
                    text_auto=False,
                    color_continuous_scale=[(0, "lightyellow"), (1e-10, "lightblue"), (1, "darkblue")],
                    )
    fig.update_layout({
        'plot_bgcolor': 'rgba(200, 200, 200, 0.5)',
        'paper_bgcolor': 'rgba(0, 0, 0, 0)',
    })
    fig.show()
    fig.update_layout(margin=dict(
        t=0, b=0, l=0, r=0
    ))
    fig.write_image("heatmap.pdf", scale=2.4)

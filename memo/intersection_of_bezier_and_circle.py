# %% ライブラリのインポート
import sympy as sy

# %% 変数の定義
# sy.var("x0 y0 x1 y1 x2 y2 x3 y3 xc yc r t x y")

x0, y0 = sy.symbols("x0 y0")
x1, y1 = sy.symbols("x1 y1")
x2, y2 = sy.symbols("x2 y2")
xc, yc = sy.symbols("xc yc")
r = sy.symbols("r")
t = sy.symbols("t")
x, y = sy.symbols("x y")

# %% ベジエ曲線の定義
x_bezier = (1 - t) ** 2 * x0 + 2 * (1 - t) * t * x1 + t**2 * x2
y_bezier = (1 - t) ** 2 * y0 + 2 * (1 - t) * t * y1 + t**2 * y2

# %% 円の定義
circle = (x - xc) ** 2 + (y - yc) ** 2 - r**2

# %% ベジエ曲線と円の交点を求める
f = circle.subs({x: x_bezier, y: y_bezier})
df = sy.diff(f, t)

print("f = ", f)
print("df = ", df)

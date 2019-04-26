print("hi maria, my love...FVIF")

r = 0.07
# time_horizon = 5
capital_gains_tax = 0.2
# cost_basis = 1.0 
income_tax_rate = 0.2

def fvif(ret, th, cgt, cb, itr):
	net_return = (1 + ret)**th * (1 - itr) + itr

	print(net_return)
	return (1 + ret)**th * (1 - itr) + itr - (1 - cb) * cgt



for cost_basis in [0.3, 1.0]:
	print("cost_basis__", cost_basis)
	for time_horizon in range(20, 21):
		print("________________")
		print("fvif", 100000 * fvif(r, time_horizon, capital_gains_tax, cost_basis, income_tax_rate))
		print("________________")

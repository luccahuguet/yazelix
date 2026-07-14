return {
	setup = function()
		if Status then
			Status:children_remove(3, Status.LEFT)
		end
	end,
}

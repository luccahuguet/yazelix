function Status:render(area)
	self.area = area

	local line = ui.Line { self:percentage(), self:position() }
	return {
		ui.Paragraph(area, { line }):align(ui.Paragraph.CENTER),
	}
end
